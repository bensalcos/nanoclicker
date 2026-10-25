//! Contain OS-managed connection/authentication work outside input ownership.
use nanoclick_core::Cancellation;
use std::{
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    thread,
    time::{Duration, Instant},
};
use x11rb::errors::ConnectError;

// DNS, socket establishment and filesystem access cannot all be interrupted by
// portable Rust APIs. Never detach an unbounded number of blocked operations.
static SETUP_JOBS: AtomicUsize = AtomicUsize::new(0);
const MAX_SETUP_JOBS: usize = 4;
const SETUP_BUDGET: Duration = Duration::from_secs(3);
struct Slot(&'static AtomicUsize);
impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}
fn failure(kind: io::ErrorKind, message: &str) -> ConnectError {
    io::Error::new(kind, message).into()
}
pub fn check(cancel: &Cancellation, deadline: Instant) -> Result<(), ConnectError> {
    if cancel.is_cancelled() {
        return Err(failure(
            io::ErrorKind::ConnectionAborted,
            "X11 setup cancelled",
        ));
    }
    if Instant::now() >= deadline {
        return Err(failure(
            io::ErrorKind::TimedOut,
            "X11 connection setup timed out",
        ));
    }
    Ok(())
}
/// The helper must only prepare connection metadata, never handshake or inject.
/// Abandoned OS work may remain blocked, but consumes one of four fixed slots.
pub fn prepare<T: Send + 'static>(
    cancel: &Cancellation,
    operation: impl FnOnce(Instant) -> Result<T, ConnectError> + Send + 'static,
) -> Result<T, ConnectError> {
    prepare_with(cancel, SETUP_BUDGET, &SETUP_JOBS, operation)
}
fn prepare_with<T: Send + 'static>(
    cancel: &Cancellation,
    budget: Duration,
    jobs: &'static AtomicUsize,
    operation: impl FnOnce(Instant) -> Result<T, ConnectError> + Send + 'static,
) -> Result<T, ConnectError> {
    let deadline = Instant::now() + budget;
    check(cancel, deadline)?;
    jobs.fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
        (count < MAX_SETUP_JOBS).then_some(count + 1)
    })
    .map_err(|_| {
        failure(
            io::ErrorKind::WouldBlock,
            "X11 connection setup is busy; earlier OS operations have not returned",
        )
    })?;
    let slot = Slot(jobs);
    let (send, receive) = mpsc::sync_channel(1);
    // Dropping a JoinHandle detaches it. The slot stays inside that thread until
    // OS work returns (or unwinds); timeout/cancellation never frees it early.
    thread::Builder::new()
        .name("nanoclick-setup".into())
        .spawn(move || {
            let _slot = slot;
            let _ = send.send(operation(deadline));
        })?;
    loop {
        check(cancel, deadline)?;
        let remaining = deadline.saturating_duration_since(Instant::now());
        match receive.recv_timeout(remaining.min(Duration::from_millis(10))) {
            Ok(result) => {
                check(cancel, deadline)?;
                return result;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                return Err(failure(
                    io::ErrorKind::Other,
                    "X11 connection setup worker stopped",
                ));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use nanoclick_core::BackendError;
    #[test]
    fn cancellation_returns_before_blocked_os_work_and_discards_late_result()
    -> Result<(), Box<dyn std::error::Error>> {
        static JOBS: AtomicUsize = AtomicUsize::new(0);
        let cancel = Cancellation::default();
        let request = cancel.clone();
        let (entered, ready) = mpsc::channel();
        let (release, blocked) = mpsc::channel();
        let (dropped, observed) = mpsc::channel();
        struct ResultGuard(mpsc::Sender<()>);
        impl Drop for ResultGuard {
            fn drop(&mut self) {
                let _ = self.0.send(());
            }
        }
        let caller = thread::spawn(move || {
            prepare_with(&request, Duration::from_secs(2), &JOBS, move |_| {
                entered.send(()).map_err(io::Error::other)?;
                blocked.recv().map_err(io::Error::other)?;
                Ok(ResultGuard(dropped))
            })
        });
        ready.recv_timeout(Duration::from_secs(1))?;
        cancel.fail(BackendError::new("Fixture cancellation"));
        let (finished, done) = mpsc::channel();
        thread::spawn(move || {
            let _ = finished.send(caller.join().map(|result| result.is_err()).unwrap_or(false));
        });
        let outcome = done.recv_timeout(Duration::from_secs(1));
        // Always release our own blocked operation before assertions.
        release.send(())?;
        assert!(outcome?);
        observed.recv_timeout(Duration::from_secs(1))?;
        Ok(())
    }
    #[test]
    fn timeout_does_not_release_slots_until_os_work_finishes()
    -> Result<(), Box<dyn std::error::Error>> {
        static JOBS: AtomicUsize = AtomicUsize::new(0);
        let mut releases = Vec::new();
        let mut finished = Vec::new();
        for _ in 0..MAX_SETUP_JOBS {
            let (release, blocked) = mpsc::channel();
            let (done, observed) = mpsc::channel();
            releases.push(release);
            finished.push(observed);
            assert!(
                prepare_with(
                    &Cancellation::default(),
                    Duration::from_millis(30),
                    &JOBS,
                    move |_| {
                        let _ = blocked.recv();
                        let _ = done.send(());
                        Ok(())
                    }
                )
                .is_err()
            );
        }
        let saturated = JOBS.load(Ordering::Acquire);
        let result: Result<(), ConnectError> = prepare_with(
            &Cancellation::default(),
            Duration::from_secs(1),
            &JOBS,
            |_| panic!("Saturated setup must not spawn"),
        );
        for release in releases {
            let _ = release.send(());
        }
        for observed in finished {
            observed.recv_timeout(Duration::from_secs(1))?;
        }
        assert_eq!(saturated, MAX_SETUP_JOBS);
        assert!(
            matches!(result, Err(ConnectError::IoError(error)) if error.kind() == io::ErrorKind::WouldBlock)
        );
        let deadline = Instant::now() + Duration::from_secs(1);
        while JOBS.load(Ordering::Acquire) != 0 && Instant::now() < deadline {
            thread::yield_now();
        }
        assert_eq!(JOBS.load(Ordering::Acquire), 0);
        assert!(
            prepare_with(
                &Cancellation::default(),
                Duration::from_secs(1),
                &JOBS,
                |_| Ok(())
            )
            .is_ok()
        );
        Ok(())
    }
}
