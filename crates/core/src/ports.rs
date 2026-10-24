use crate::MouseButton;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

struct CancellationState {
    cancelled: AtomicBool,
    failure: Mutex<Option<BackendError>>,
    waiter_wake: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

/// Cooperative cancellation and asynchronous native-session failure signal.
#[derive(Clone)]
pub struct Cancellation(Arc<CancellationState>);
impl Default for Cancellation {
    fn default() -> Self {
        Self(Arc::new(CancellationState {
            cancelled: AtomicBool::new(false),
            failure: Mutex::new(None),
            waiter_wake: Mutex::new(None),
        }))
    }
}
impl Cancellation {
    /// True once Stop, Shutdown, or a native failure was requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled.load(Ordering::Acquire)
    }
    pub(crate) fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::Release);
        let wake = self
            .0
            .waiter_wake
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if let Some(wake) = wake {
            wake();
        }
    }
    /// Installs this run's single waiter notification. Only the worker's Waiter
    /// may replace it. The callback must be nonblocking, idempotent and must not
    /// call back into Engine or Cancellation. It may run on any thread.
    /// Already-cancelled tokens notify immediately; races may notify twice.
    pub fn install_waiter_wake(&self, wake: impl Fn() + Send + Sync + 'static) {
        let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(wake);
        *self
            .0
            .waiter_wake
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(wake.clone());
        if self.is_cancelled() {
            wake();
        }
    }
    pub(crate) fn clear_waiter_wake(&self) {
        self.0
            .waiter_wake
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
    /// Reports a native revocation/disconnection and wakes the worker immediately.
    pub fn fail(&self, error: BackendError) {
        let mut failure = self
            .0
            .failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if failure.is_none() {
            *failure = Some(error);
        }
        drop(failure);
        self.cancel();
    }
    pub(crate) fn take_failure(&self) -> Option<BackendError> {
        self.0
            .failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}

/// A native operation failed. Messages must be suitable for the English UI.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct BackendError {
    message: String,
}
impl BackendError {
    /// Builds an error at the platform boundary, outside successful hot paths.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// A prepared input session, owned exclusively by the execution worker.
///
/// `click` must send a complete down/up pair or return an error, retaining
/// enough state for `finish` to attempt release. Native operations must have
/// bounded timeouts; preparation must observe its run's cancellation token.
pub trait InputSink: 'static {
    /// Sends one pair. Success does not prove receipt by the target application.
    fn click(&mut self, button: MouseButton) -> Result<(), BackendError>;
    /// Releases pending buttons and closes the native session.
    fn finish(&mut self) -> Result<(), BackendError> {
        Ok(())
    }
}

/// A worker-owned, cancelable monotonic wait; no GUI types or busy waiting.
/// Implementations may return early. Engine rechecks the deadline and token.
pub trait Waiter: Send + 'static {
    /// Resets native state and attaches this run's cancellation notification.
    fn bind(&mut self, token: &Cancellation) -> Result<(), BackendError>;
    /// Blocks until a deadline or cancellation, with bounded native operations.
    fn wait_until(
        &mut self,
        deadline: std::time::Instant,
        token: &Cancellation,
    ) -> Result<(), BackendError>;
    /// Disarms timers on every terminal path, including preparation failure.
    fn finish(&mut self) -> Result<(), BackendError> {
        Ok(())
    }
}
