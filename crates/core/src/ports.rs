use crate::MouseButton;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

struct CancellationState {
    cancelled: AtomicBool,
    failure: Mutex<Option<BackendError>>,
    notify: Box<dyn Fn() + Send + Sync>,
}

/// Cooperative cancellation and asynchronous native-session failure signal.
#[derive(Clone)]
pub struct Cancellation(Arc<CancellationState>);
impl Default for Cancellation {
    fn default() -> Self {
        Self::with_notify(|| {})
    }
}
impl Cancellation {
    pub(crate) fn with_notify(notify: impl Fn() + Send + Sync + 'static) -> Self {
        Self(Arc::new(CancellationState {
            cancelled: AtomicBool::new(false),
            failure: Mutex::new(None),
            notify: Box::new(notify),
        }))
    }
    /// True once Stop, Shutdown, or a native failure was requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled.load(Ordering::Acquire)
    }
    pub(crate) fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::Release);
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
        self.cancel();
        drop(failure);
        (self.0.notify)();
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
