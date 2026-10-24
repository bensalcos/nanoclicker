use crate::{BackendError, Cancellation, Waiter};
use std::{
    sync::{Arc, Condvar, Mutex},
    time::Instant,
};

/// Portable blocking wait. The predicate and notification share a separate
/// mutex so cancellation cannot be lost between checking and parking.
#[derive(Default)]
pub struct StdWaiter {
    signal: Arc<WaitSignal>,
}
#[derive(Default)]
struct WaitSignal {
    mutex: Mutex<()>,
    changed: Condvar,
}
impl Waiter for StdWaiter {
    fn bind(&mut self, token: &Cancellation) -> Result<(), BackendError> {
        let signal = self.signal.clone();
        token.install_waiter_wake(move || {
            let _guard = signal
                .mutex
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            signal.changed.notify_one();
        });
        Ok(())
    }
    fn wait_until(&mut self, deadline: Instant, token: &Cancellation) -> Result<(), BackendError> {
        let mut guard = self
            .signal
            .mutex
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        loop {
            if token.is_cancelled() {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Ok(());
            }
            (guard, _) = self
                .signal
                .changed
                .wait_timeout(guard, remaining)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }
}
