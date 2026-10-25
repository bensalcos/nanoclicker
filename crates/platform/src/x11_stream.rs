//! Protocol deadlines for the worker-owned X11 connection. No shared GUI stream.
use nanoclick_core::Cancellation;
use rustix::event::{PollFd, PollFlags, Timespec, poll};
use std::{
    cell::Cell,
    io::{self, IoSlice},
    os::fd::{AsFd, BorrowedFd},
    time::{Duration, Instant},
};
use x11rb::{
    errors::ConnectError,
    reexports::x11rb_protocol::{parse_display, xauth},
    rust_connection::{DefaultStream, PollMode, RustConnection, Stream},
    utils::RawFdContainer,
};

pub type X11Connection = RustConnection<DeadlineStream>;

/// Preserve upstream address/authentication semantics without blocking the caller
/// on OS-managed setup. Only this caller owns protocol handshake and input.
pub fn connect(cancel: &Cancellation) -> Result<(X11Connection, usize), ConnectError> {
    let request = cancel.clone();
    let (stream, screen, name, data) = super::x11_setup::prepare(cancel, move |deadline| {
        let display = parse_display::parse_display(None)?;
        let screen = usize::from(display.screen);
        let mut last_error = None;
        for address in display.connect_instruction() {
            super::x11_setup::check(&request, deadline)?;
            match DefaultStream::connect(&address) {
                Ok((stream, (family, peer))) => {
                    super::x11_setup::check(&request, deadline)?;
                    let (name, data) = xauth::get_auth(family, &peer, display.display)
                        .ok()
                        .flatten()
                        .unwrap_or_default();
                    super::x11_setup::check(&request, deadline)?;
                    return Ok((stream, screen, name, data));
                }
                Err(error) => last_error = Some(error),
            }
        }
        Err(ConnectError::IoError(last_error.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "No usable X11 display address")
        })))
    })?;
    RustConnection::connect_to_stream_with_auth_info(
        DeadlineStream::new(stream, cancel.clone()),
        screen,
        name,
        data,
    )
    .map(|connection| (connection, screen))
}

pub struct DeadlineStream {
    inner: DefaultStream,
    cancel: Cancellation,
    deadline: Cell<Instant>,
    preparation: Cell<bool>,
    cancel_deadline: Cell<Option<Instant>>,
}
impl DeadlineStream {
    pub fn new(inner: DefaultStream, cancel: Cancellation) -> Self {
        Self {
            inner,
            cancel,
            deadline: Cell::new(Instant::now() + Duration::from_secs(3)),
            preparation: Cell::new(true),
            cancel_deadline: Cell::new(None),
        }
    }
    fn begin(&self, timeout: Duration, preparation: bool) {
        self.deadline.set(Instant::now() + timeout);
        self.preparation.set(preparation);
        self.cancel_deadline.set(None);
    }
    pub fn begin_request(&self) {
        // A sent pair cannot be retracted. Let it complete, or report that
        // delivery/release could not be verified; cleanup has its own bound.
        // After Stop each exchange receives 25 ms grace, keeping all three
        // possible attempts (pair, finish, Drop) short without skipping release.
        self.begin(Duration::from_millis(250), false);
    }
    fn remaining(&self) -> io::Result<Duration> {
        let now = Instant::now();
        let mut deadline = self.deadline.get();
        if self.cancel.is_cancelled() {
            if self.preparation.get() {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "X11 preparation cancelled",
                ));
            }
            let grace = self.cancel_deadline.get().unwrap_or_else(|| {
                let deadline = now + Duration::from_millis(25);
                self.cancel_deadline.set(Some(deadline));
                deadline
            });
            deadline = deadline.min(grace);
        }
        deadline
            .checked_duration_since(now)
            .filter(|value| !value.is_zero())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::TimedOut, "The X server response timed out")
            })
    }
}
impl AsFd for DeadlineStream {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}
impl Stream for DeadlineStream {
    fn poll(&self, mode: PollMode) -> io::Result<()> {
        let mut flags = PollFlags::empty();
        if mode.readable() {
            flags |= PollFlags::IN;
        }
        if mode.writable() {
            flags |= PollFlags::OUT;
        }
        let mut descriptors = [PollFd::new(&self.inner, flags)];
        loop {
            let remaining = self.remaining()?;
            // Observe Stop during blocked I/O without replacing the waiter's
            // single notification hook. Healthy ready I/O completes in one poll.
            let interval = remaining.min(Duration::from_millis(10));
            let timeout = Timespec::try_from(interval).map_err(io::Error::other)?;
            match poll(&mut descriptors, Some(&timeout)) {
                Ok(0) => continue,
                Ok(_) => {
                    self.remaining()?;
                    return Ok(());
                }
                Err(rustix::io::Errno::INTR) => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }
    fn read(&self, buffer: &mut [u8], fds: &mut Vec<RawFdContainer>) -> io::Result<usize> {
        self.remaining()?;
        self.inner.read(buffer, fds)
    }
    fn write(&self, buffer: &[u8], fds: &mut Vec<RawFdContainer>) -> io::Result<usize> {
        self.remaining()?;
        self.inner.write(buffer, fds)
    }
    fn write_vectored(
        &self,
        buffers: &[IoSlice<'_>],
        fds: &mut Vec<RawFdContainer>,
    ) -> io::Result<usize> {
        self.remaining()?;
        self.inner.write_vectored(buffers, fds)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use nanoclick_core::BackendError;
    use std::{io::Write, os::unix::net::UnixStream};
    #[test]
    fn expired_deadline_rejects_io_even_when_the_peer_is_ready() -> io::Result<()> {
        let (local, mut peer) = UnixStream::pair()?;
        peer.write_all(b"x")?;
        let stream = DeadlineStream::new(
            DefaultStream::from_unix_stream(local)?.0,
            Cancellation::default(),
        );
        stream.begin(Duration::ZERO, false);
        assert!(
            matches!(stream.poll(PollMode::Readable), Err(error) if error.kind()==io::ErrorKind::TimedOut)
        );
        assert!(
            matches!(stream.read(&mut [0], &mut Vec::new()), Err(error) if error.kind()==io::ErrorKind::TimedOut)
        );
        assert!(
            matches!(stream.write(b"x", &mut Vec::new()), Err(error) if error.kind()==io::ErrorKind::TimedOut)
        );
        Ok(())
    }
    #[test]
    fn preparation_observes_cancel_but_recovery_can_still_attempt_io() -> io::Result<()> {
        let (local, mut peer) = UnixStream::pair()?;
        peer.write_all(b"x")?;
        let cancel = Cancellation::default();
        let stream = DeadlineStream::new(DefaultStream::from_unix_stream(local)?.0, cancel.clone());
        cancel.fail(BackendError::new("Fixture cancellation"));
        assert!(
            matches!(stream.poll(PollMode::Readable), Err(error) if error.kind()==io::ErrorKind::ConnectionAborted)
        );
        stream.begin_request();
        stream.poll(PollMode::Readable)?;
        assert_eq!(stream.read(&mut [0], &mut Vec::new())?, 1);
        Ok(())
    }
}
