//! High-resolution, one-shot native timer interrupted by a per-run cancel event.
use nanoclick_core::{BackendError, Cancellation, Waiter};
use std::{
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr::null,
    sync::Arc,
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::{WAIT_FAILED, WAIT_OBJECT_0},
    System::Threading::{
        CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CancelWaitableTimer, CreateEventW,
        CreateWaitableTimerExW, INFINITE, SYNCHRONIZATION_SYNCHRONIZE, SetEvent,
        SetWaitableTimerEx, TIMER_MODIFY_STATE, WaitForMultipleObjects,
    },
};

pub(crate) struct WindowsWaiter {
    timer: OwnedHandle,
    cancel: Option<Arc<OwnedHandle>>,
    armed: bool,
}
fn error(operation: &str) -> BackendError {
    BackendError::new(format!(
        "Windows timer {operation}: {}",
        std::io::Error::last_os_error()
    ))
}
impl WindowsWaiter {
    pub(crate) fn new() -> Result<Self, BackendError> {
        // SAFETY: unnamed non-inheritable timer, default security. High-resolution
        // mode is supported by the Windows 11 baseline; no global timer setting.
        let timer = unsafe {
            CreateWaitableTimerExW(
                null(),
                null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_MODIFY_STATE | SYNCHRONIZATION_SYNCHRONIZE,
            )
        };
        if timer.is_null() {
            return Err(error("creation failed"));
        }
        // SAFETY: a newly allocated handle, transferred into a single RAII owner.
        let timer = unsafe { OwnedHandle::from_raw_handle(timer) };
        Ok(Self {
            timer,
            cancel: None,
            armed: false,
        })
    }
}
impl Waiter for WindowsWaiter {
    fn bind(&mut self, token: &Cancellation) -> Result<(), BackendError> {
        // A fresh event per run prevents an old callback, already in flight when
        // its token was cleared, from signalling a later run's cancellation.
        // SAFETY: unnamed, non-inheritable, initially clear manual-reset event.
        let handle = unsafe { CreateEventW(null(), 1, 0, null()) };
        if handle.is_null() {
            return Err(error("cancellation event creation failed"));
        }
        // SAFETY: new owned handle; Arc preserves lifetime through callbacks.
        let cancel = Arc::new(unsafe { OwnedHandle::from_raw_handle(handle) });
        self.cancel = Some(cancel.clone());
        token.install_waiter_wake(move || {
            // SAFETY: the captured Arc owns a valid manual-reset event throughout
            // this callback, including cancellation racing with terminal cleanup.
            unsafe {
                SetEvent(cancel.as_raw_handle());
            }
        });
        Ok(())
    }
    fn wait_until(&mut self, deadline: Instant, token: &Cancellation) -> Result<(), BackendError> {
        if token.is_cancelled() || Instant::now() >= deadline {
            return Ok(());
        }
        let due =
            crate::windows_time::relative_due(deadline.saturating_duration_since(Instant::now()))?;
        // SAFETY: valid owned timer and stack due time copied by Windows; no APC,
        // no resume from sleep, no periodic timer, zero tolerance requested.
        if unsafe {
            SetWaitableTimerEx(self.timer.as_raw_handle(), &due, 0, None, null(), null(), 0)
        } == 0
        {
            return Err(error("arming failed"));
        }
        self.armed = true;
        let cancel = self
            .cancel
            .as_ref()
            .ok_or_else(|| BackendError::new("Windows timer was not bound to a run."))?;
        let handles = [cancel.as_raw_handle(), self.timer.as_raw_handle()];
        // SAFETY: both handles remain owned throughout this synchronous wait.
        // Lowest index wins when both signal, giving cancellation priority.
        match unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, INFINITE) } {
            WAIT_OBJECT_0 => Ok(()),
            value if value == WAIT_OBJECT_0 + 1 => Ok(()),
            WAIT_FAILED => Err(error("wait failed")),
            value => Err(BackendError::new(format!(
                "Unexpected Windows timer wait result: {value}."
            ))),
        }
    }
    fn finish(&mut self) -> Result<(), BackendError> {
        if self.armed {
            // SAFETY: valid owned timer; only this worker arms/disarms it.
            if unsafe { CancelWaitableTimer(self.timer.as_raw_handle()) } == 0 {
                return Err(error("disarming failed"));
            }
            self.armed = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, thread, time::Duration};
    use windows_sys::Win32::{Foundation::WAIT_TIMEOUT, System::Threading::WaitForSingleObject};

    #[test]
    fn an_old_run_cannot_signal_the_next_runs_event() -> Result<(), BackendError> {
        let mut waiter = WindowsWaiter::new()?;
        let old = Cancellation::default();
        waiter.bind(&old)?;
        let current = Cancellation::default();
        waiter.bind(&current)?;
        old.fail(BackendError::new("late old session failure"));
        let handle = waiter
            .cancel
            .as_ref()
            .ok_or_else(|| BackendError::new("Missing test event"))?;
        // SAFETY: owned event remains valid; zero timeout is a nonblocking probe.
        assert_eq!(
            unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) },
            WAIT_TIMEOUT
        );
        current.fail(BackendError::new("current session failure"));
        // SAFETY: same valid event, now expected to have been signalled.
        assert_eq!(
            unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) },
            WAIT_OBJECT_0
        );
        waiter.finish()
    }
    #[test]
    fn native_cancel_event_interrupts_a_long_timer() -> Result<(), Box<dyn std::error::Error>> {
        let token = Cancellation::default();
        let worker_token = token.clone();
        let mut waiter = WindowsWaiter::new()?;
        waiter.bind(&token)?;
        let (done, completed) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || -> Result<(), BackendError> {
            waiter.wait_until(Instant::now() + Duration::from_secs(60), &worker_token)?;
            waiter.finish()?;
            let _ = done.send(());
            Ok(())
        });
        token.fail(BackendError::new("test cancellation; no input is sent"));
        completed.recv_timeout(Duration::from_secs(3))?;
        worker.join().map_err(|_| "Timer test worker panicked")??;
        Ok(())
    }
}
