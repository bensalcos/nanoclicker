#![expect(clippy::unwrap_used, reason = "Test fixture preconditions.")]
use nanoclick_core::{
    BackendError, Cancellation, ClickConfig, Engine, InputSink, MouseButton, RunState, Waiter,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

struct Counter(Arc<AtomicU64>);
impl InputSink for Counter {
    fn click(&mut self, _: MouseButton) -> Result<(), BackendError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
struct FailedWait {
    finish: Arc<AtomicU64>,
}
impl Waiter for FailedWait {
    fn bind(&mut self, _: &Cancellation) -> Result<(), BackendError> {
        Ok(())
    }
    fn wait_until(&mut self, _: Instant, _: &Cancellation) -> Result<(), BackendError> {
        Err(BackendError::new("native timer failed"))
    }
    fn finish(&mut self) -> Result<(), BackendError> {
        self.finish.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
#[test]
fn timer_failure_is_terminal_and_cleanup_runs() {
    let clicks = Arc::new(AtomicU64::new(0));
    let shared = clicks.clone();
    let finishes = Arc::new(AtomicU64::new(0));
    let engine = Engine::with_waiter(
        move |_| Ok(Counter(shared.clone())),
        FailedWait {
            finish: finishes.clone(),
        },
        || {},
    )
    .unwrap();
    engine
        .start(ClickConfig::new(999999, MouseButton::Left, None).unwrap())
        .unwrap();
    let until = Instant::now() + Duration::from_secs(3);
    while engine.snapshot().state != RunState::Faulted {
        assert!(Instant::now() < until);
        thread::yield_now();
    }
    assert_eq!(clicks.load(Ordering::Relaxed), 1);
    assert_eq!(finishes.load(Ordering::Relaxed), 1);
    assert!(
        engine
            .snapshot()
            .error
            .unwrap()
            .contains("native timer failed")
    );
}
#[test]
fn attaching_waiter_wake_after_cancellation_still_notifies() {
    let token = Cancellation::default();
    token.fail(BackendError::new("revoked before binding"));
    let wakes = Arc::new(AtomicU64::new(0));
    let shared = wakes.clone();
    token.install_waiter_wake(move || {
        shared.fetch_add(1, Ordering::Relaxed);
    });
    assert_eq!(wakes.load(Ordering::Relaxed), 1);
}

struct FailedBind(Arc<AtomicU64>);
impl Waiter for FailedBind {
    fn bind(&mut self, _: &Cancellation) -> Result<(), BackendError> {
        Err(BackendError::new("binding failed"))
    }
    fn wait_until(&mut self, _: Instant, _: &Cancellation) -> Result<(), BackendError> {
        panic!("A failed bind must never enter a wait")
    }
    fn finish(&mut self) -> Result<(), BackendError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Err(BackendError::new("disarm failed"))
    }
}
#[test]
fn binding_failure_skips_input_preparation_and_preserves_cleanup_failure() {
    let prepares = Arc::new(AtomicU64::new(0));
    let shared = prepares.clone();
    let finishes = Arc::new(AtomicU64::new(0));
    let engine = Engine::with_waiter(
        move |_| {
            shared.fetch_add(1, Ordering::Relaxed);
            Ok(Counter(shared.clone()))
        },
        FailedBind(finishes.clone()),
        || {},
    )
    .unwrap();
    engine.start(ClickConfig::default()).unwrap();
    let until = Instant::now() + Duration::from_secs(3);
    while engine.snapshot().state != RunState::Faulted {
        assert!(Instant::now() < until);
        thread::yield_now();
    }
    assert_eq!(prepares.load(Ordering::Relaxed), 0);
    assert_eq!(finishes.load(Ordering::Relaxed), 1);
    let error = engine.snapshot().error.unwrap();
    assert!(error.contains("binding failed") && error.contains("disarm failed"));
}
#[test]
fn portable_waiter_does_not_lose_cancellation_when_parking() {
    for _ in 0..50 {
        let token = Cancellation::default();
        let worker_token = token.clone();
        let (ready, waiting) = std::sync::mpsc::sync_channel(1);
        let (done, completed) = std::sync::mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            let mut waiter = nanoclick_core::StdWaiter::default();
            waiter.bind(&worker_token).unwrap();
            ready.send(()).unwrap();
            waiter
                .wait_until(Instant::now() + Duration::from_secs(30), &worker_token)
                .unwrap();
            done.send(()).unwrap();
        });
        waiting.recv_timeout(Duration::from_secs(1)).unwrap();
        token.fail(BackendError::new("native disconnect"));
        completed.recv_timeout(Duration::from_secs(1)).unwrap();
        worker.join().unwrap();
    }
}

struct CancelDuringWait;
impl Waiter for CancelDuringWait {
    fn bind(&mut self, _: &Cancellation) -> Result<(), BackendError> {
        Ok(())
    }
    fn wait_until(&mut self, _: Instant, token: &Cancellation) -> Result<(), BackendError> {
        token.fail(BackendError::new("revoked during native wait"));
        Ok(())
    }
}
#[test]
fn waking_rechecks_cancellation_before_injecting_again() {
    let clicks = Arc::new(AtomicU64::new(0));
    let shared = clicks.clone();
    let engine = Engine::with_waiter(
        move |_| Ok(Counter(shared.clone())),
        CancelDuringWait,
        || {},
    )
    .unwrap();
    engine
        .start(ClickConfig::new(999999, MouseButton::Left, Some(2)).unwrap())
        .unwrap();
    let until = Instant::now() + Duration::from_secs(3);
    while engine.snapshot().state.is_active() {
        assert!(Instant::now() < until);
        thread::yield_now();
    }
    assert_eq!(engine.snapshot().state, RunState::Faulted);
    assert_eq!(clicks.load(Ordering::Relaxed), 1);
    assert!(
        engine
            .snapshot()
            .error
            .unwrap()
            .contains("revoked during native wait")
    );
}
