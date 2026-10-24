use crate::{
    BackendError, Cancellation, ClickConfig, InputSink, Metrics, RunState, Scheduler, Snapshot,
    StopCondition, StopReason,
};
use std::{
    sync::{Arc, Condvar, Mutex, MutexGuard},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

struct Control {
    snapshot: Snapshot,
    pending: Option<(ClickConfig, Duration)>,
    cancellation: Cancellation,
    shutdown: bool,
}
struct Shared {
    control: Mutex<Control>,
    changed: Condvar,
    wake: Box<dyn Fn() + Send + Sync>,
}
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // Poison must not prevent shutdown/release after an unrelated UI panic.
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Errors creating a worker or allocating a new run identity.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Could not create the click worker: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("No more execution identifiers are available. Restart NanoClick.")]
    IdentityExhausted,
    #[error("Preparation delay must not exceed 60 seconds.")]
    DelayOutOfRange,
}

/// One persistent worker, a bounded one-slot Start mailbox, and priority Stop.
/// No UI or platform-specific types cross this boundary.
pub struct Engine {
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
}
impl Engine {
    /// Creates a worker. The factory prepares native resources on that worker.
    /// `wake` must be nonblocking and only request a UI refresh, never reenter.
    pub fn new<S, F, W>(factory: F, wake: W) -> Result<Self, EngineError>
    where
        S: InputSink,
        F: Fn(&Cancellation) -> Result<S, BackendError> + Send + 'static,
        W: Fn() + Send + Sync + 'static,
    {
        let shared = Arc::new(Shared {
            control: Mutex::new(Control {
                snapshot: Snapshot::default(),
                pending: None,
                cancellation: Cancellation::default(),
                shutdown: false,
            }),
            changed: Condvar::new(),
            wake: Box::new(wake),
        });
        let worker_shared = shared.clone();
        let worker = thread::Builder::new()
            .name("nanoclick-worker".into())
            .spawn(move || worker_loop(worker_shared, factory))?;
        Ok(Self {
            shared,
            worker: Some(worker),
        })
    }
    /// Starts a validated plan. Repeated Start while active returns the same ID.
    pub fn start(&self, plan: ClickConfig) -> Result<u64, EngineError> {
        self.start_with_delay(plan, Duration::ZERO)
    }
    /// Starts after backend preparation and a cancelable delay (up to 60 s).
    /// Permission dialogs complete before the delay begins.
    pub fn start_with_delay(&self, plan: ClickConfig, delay: Duration) -> Result<u64, EngineError> {
        if delay > Duration::from_secs(60) {
            return Err(EngineError::DelayOutOfRange);
        }
        let mut control = lock(&self.shared.control);
        if control.snapshot.state.is_active() {
            return Ok(control.snapshot.run_id);
        }
        let id = control
            .snapshot
            .run_id
            .checked_add(1)
            .ok_or(EngineError::IdentityExhausted)?;
        control.snapshot = Snapshot {
            run_id: id,
            state: RunState::Preparing,
            ..Snapshot::default()
        };
        let weak = Arc::downgrade(&self.shared);
        control.cancellation = Cancellation::with_notify(move || {
            if let Some(shared) = weak.upgrade() {
                let _guard = lock(&shared.control);
                shared.changed.notify_one();
            }
        });
        control.pending = Some((plan, delay));
        self.shared.changed.notify_one();
        drop(control);
        (self.shared.wake)();
        Ok(id)
    }
    /// Requests Stop without waiting for the worker. Inspect the snapshot for ack.
    pub fn stop(&self) {
        let mut control = lock(&self.shared.control);
        if !control.snapshot.state.is_active() {
            return;
        }
        control.cancellation.cancel();
        control.snapshot.state = RunState::Stopping;
        self.shared.changed.notify_one();
        drop(control);
        (self.shared.wake)();
    }
    /// Copies the latest small snapshot, preserving terminal outcomes.
    pub fn snapshot(&self) -> Snapshot {
        lock(&self.shared.control).snapshot.clone()
    }
}
impl Drop for Engine {
    fn drop(&mut self) {
        {
            let mut control = lock(&self.shared.control);
            control.shutdown = true;
            control.cancellation.cancel();
            self.shared.changed.notify_one();
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn worker_loop<S: InputSink>(
    shared: Arc<Shared>,
    factory: impl Fn(&Cancellation) -> Result<S, BackendError>,
) {
    loop {
        let (plan, delay, token) = {
            let mut control = lock(&shared.control);
            while control.pending.is_none() && !control.shutdown {
                control = shared
                    .changed
                    .wait(control)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
            if control.shutdown {
                return;
            }
            let Some((plan, delay)) = control.pending.take() else {
                continue;
            };
            (plan, delay, control.cancellation.clone())
        };
        let mut metrics = Metrics::default();
        // Isolate panics from external adapters. Their RAII cleanup runs while
        // unwinding; a failed run must not leave the UI stuck in Running.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| if token.is_cancelled() {
            Ok(StopReason::Stopped)
        } else {
            match factory(&token) {
                Err(error) if token.is_cancelled() => {
                    let _ = error;
                    Ok(StopReason::Stopped)
                }
                Err(error) => Err(error),
                Ok(mut sink) => {
                    let result = execute(&shared, &token, plan, delay, &mut sink, &mut metrics);
                    match (result, sink.finish()) {
                        (Err(first), Err(cleanup)) => {
                            Err(BackendError::new(format!("{first} Cleanup: {cleanup}")))
                        }
                        (_, Err(cleanup)) => Err(cleanup),
                        (result, Ok(())) => result,
                    }
                }
            }
        })).unwrap_or_else(|_| Err(BackendError::new("The native input backend ended unexpectedly. Restart the session or the application.")));
        let result = match (token.take_failure(), result) {
            (Some(first), Err(cleanup)) => {
                Err(BackendError::new(format!("{first} Session: {cleanup}")))
            }
            (Some(error), _) => Err(error),
            (None, result) => result,
        };
        let mut control = lock(&shared.control);
        control.snapshot.metrics = metrics;
        control.snapshot.ready_at = None;
        match result {
            Ok(reason) => {
                control.snapshot.state = RunState::Idle;
                control.snapshot.reason = Some(reason);
            }
            Err(error) => {
                control.snapshot.state = RunState::Faulted;
                control.snapshot.reason = Some(StopReason::Failed);
                control.snapshot.error = Some(error.to_string());
            }
        }
        drop(control);
        (shared.wake)();
    }
}

fn execute(
    shared: &Shared,
    token: &Cancellation,
    plan: ClickConfig,
    delay: Duration,
    sink: &mut impl InputSink,
    metrics: &mut Metrics,
) -> Result<StopReason, BackendError> {
    if token.is_cancelled() || !prepare_delay(shared, token, delay)? {
        return Ok(StopReason::Stopped);
    }
    {
        let mut control = lock(&shared.control);
        if token.is_cancelled() {
            return Ok(StopReason::Stopped);
        }
        control.snapshot.ready_at = None;
        control.snapshot.state = RunState::Running;
    }
    (shared.wake)();
    let epoch = Instant::now();
    let mut publish = epoch;
    let mut scheduler = Scheduler::new(plan.interval());
    loop {
        // Predicate and notification use the same mutex: Stop cannot be lost
        // between checking cancellation and parking in the condition variable.
        let mut control = lock(&shared.control);
        loop {
            if token.is_cancelled() {
                return Ok(StopReason::Stopped);
            }
            let remaining = scheduler.deadline().saturating_sub(epoch.elapsed());
            if remaining.is_zero() {
                break;
            }
            let (guard, _) = shared
                .changed
                .wait_timeout(control, remaining)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            control = guard;
        }
        drop(control);
        let lateness = epoch.elapsed().saturating_sub(scheduler.deadline());
        metrics.max_lateness = metrics.max_lateness.max(lateness);
        metrics.attempts = metrics
            .attempts
            .checked_add(1)
            .ok_or_else(|| BackendError::new("Attempt counter overflow."))?;
        if let Err(error) = sink.click(plan.button()) {
            metrics.failures += 1;
            return Err(error);
        }
        metrics.sent += 1;
        if matches!(plan.stop_condition(), StopCondition::Count(count) if metrics.sent >= count.get())
        {
            return Ok(StopReason::Completed);
        }
        metrics.skipped = metrics
            .skipped
            .checked_add(
                scheduler
                    .advance(epoch.elapsed())
                    .map_err(|error| BackendError::new(error.to_string()))?,
            )
            .ok_or_else(|| BackendError::new("Skipped-period counter overflow."))?;
        if metrics.sent == 1 || publish.elapsed() >= Duration::from_millis(500) {
            lock(&shared.control).snapshot.metrics = *metrics;
            (shared.wake)();
            publish = Instant::now();
        }
    }
}

// Called only after a native session exists: permission interaction cannot
// consume the user's time to move the pointer to its actual target.
fn prepare_delay(
    shared: &Shared,
    token: &Cancellation,
    delay: Duration,
) -> Result<bool, BackendError> {
    if delay.is_zero() {
        return Ok(!token.is_cancelled());
    }
    let deadline = Instant::now()
        .checked_add(delay)
        .ok_or_else(|| BackendError::new("Preparation deadline overflow."))?;
    {
        let mut control = lock(&shared.control);
        if token.is_cancelled() {
            return Ok(false);
        }
        control.snapshot.ready_at = Some(deadline);
    }
    (shared.wake)();
    let mut control = lock(&shared.control);
    loop {
        if token.is_cancelled() {
            return Ok(false);
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(true);
        }
        let (guard, _) = shared
            .changed
            .wait_timeout(control, remaining)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        control = guard;
    }
}
