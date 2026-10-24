#![expect(
    clippy::unwrap_used,
    reason = "Test fixtures fail immediately when their preconditions are not met."
)]
use nanoclick_core::{
    BackendError, Cancellation, ClickConfig, Engine, InputSink, MouseButton, RunState, StopReason,
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

fn wait_until(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !predicate() {
        assert!(
            Instant::now() < deadline,
            "worker did not reach expected state"
        );
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn completes_exact_count_and_preserves_terminal_snapshot() {
    let count = Arc::new(AtomicU64::new(0));
    let shared = count.clone();
    let engine = Engine::new(move |_: &Cancellation| Ok(Counter(shared.clone())), || {}).unwrap();
    engine
        .start(ClickConfig::new(1, MouseButton::Right, Some(10)).unwrap())
        .unwrap();
    wait_until(|| engine.snapshot().reason == Some(StopReason::Completed));
    assert_eq!(count.load(Ordering::Relaxed), 10);
    assert_eq!(engine.snapshot().state, RunState::Idle);
    assert_eq!(engine.snapshot().metrics.sent, 10);
    let metrics = engine.snapshot().metrics;
    assert!(metrics.lateness_p50 <= metrics.lateness_p95);
    assert!(metrics.lateness_p95 <= metrics.lateness_p99);
    assert!(metrics.lateness_p99 <= metrics.max_lateness);
}

#[test]
fn stop_interrupts_long_interval_and_restart_has_new_identity() {
    let count = Arc::new(AtomicU64::new(0));
    let shared = count.clone();
    let engine = Engine::new(move |_: &Cancellation| Ok(Counter(shared.clone())), || {}).unwrap();
    let first = engine
        .start(ClickConfig::new(999_999, MouseButton::Left, None).unwrap())
        .unwrap();
    wait_until(|| count.load(Ordering::Relaxed) == 1);
    assert_eq!(engine.start(ClickConfig::default()).unwrap(), first);
    engine.stop();
    wait_until(|| engine.snapshot().state == RunState::Idle);
    assert_eq!(engine.snapshot().reason, Some(StopReason::Stopped));
    let second = engine
        .start(ClickConfig::new(1, MouseButton::Left, Some(1)).unwrap())
        .unwrap();
    assert!(second > first);
    wait_until(|| engine.snapshot().reason == Some(StopReason::Completed));
    assert_eq!(count.load(Ordering::Relaxed), 2);
}

#[test]
fn cancellation_during_preparation_prevents_all_clicks() {
    let count = Arc::new(AtomicU64::new(0));
    let shared = count.clone();
    let engine = Engine::new(
        move |cancel: &Cancellation| {
            while !cancel.is_cancelled() {
                thread::sleep(Duration::from_millis(1));
            }
            Ok(Counter(shared.clone()))
        },
        || {},
    )
    .unwrap();
    engine.start(ClickConfig::default()).unwrap();
    engine.stop();
    wait_until(|| engine.snapshot().state == RunState::Idle);
    assert_eq!(count.load(Ordering::Relaxed), 0);
}

#[test]
fn reports_injection_and_cleanup_failures_without_counting_failed_pair() {
    struct Failed;
    impl InputSink for Failed {
        fn click(&mut self, _: MouseButton) -> Result<(), BackendError> {
            Err(BackendError::new("injection failed"))
        }
        fn finish(&mut self) -> Result<(), BackendError> {
            Err(BackendError::new("release failed"))
        }
    }
    let engine = Engine::new(|_: &Cancellation| Ok(Failed), || {}).unwrap();
    engine.start(ClickConfig::default()).unwrap();
    wait_until(|| engine.snapshot().state == RunState::Faulted);
    let snapshot = engine.snapshot();
    assert_eq!(snapshot.metrics.sent, 0);
    assert_eq!(snapshot.metrics.failures, 1);
    assert!(
        snapshot
            .error
            .as_deref()
            .unwrap()
            .contains("release failed")
    );
}

#[test]
fn native_revocation_wakes_a_long_wait_and_reports_fault() {
    let count = Arc::new(AtomicU64::new(0));
    let shared = count.clone();
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    let engine = Engine::new(
        move |token: &Cancellation| {
            send.send(token.clone()).unwrap();
            Ok(Counter(shared.clone()))
        },
        || {},
    )
    .unwrap();
    engine
        .start(ClickConfig::new(999_999, MouseButton::Left, None).unwrap())
        .unwrap();
    let token = receive.recv_timeout(Duration::from_secs(2)).unwrap();
    wait_until(|| count.load(Ordering::Relaxed) == 1);
    token.fail(BackendError::new("Permission revoked."));
    wait_until(|| engine.snapshot().state == RunState::Faulted);
    assert!(
        engine
            .snapshot()
            .error
            .unwrap()
            .contains("Permission revoked")
    );
}

#[test]
fn rapid_start_stop_cycles_never_lose_terminal_acknowledgement() {
    let count = Arc::new(AtomicU64::new(0));
    let engine = Engine::new(move |_: &Cancellation| Ok(Counter(count.clone())), || {}).unwrap();
    for _ in 0..200 {
        engine.start(ClickConfig::default()).unwrap();
        engine.stop();
        wait_until(|| engine.snapshot().state == RunState::Idle);
    }
    assert_eq!(engine.snapshot().run_id, 200);
}

#[test]
fn unexpected_backend_panic_is_reported_and_worker_can_retry() {
    struct Panics;
    impl InputSink for Panics {
        fn click(&mut self, _: MouseButton) -> Result<(), BackendError> {
            panic!("broken external adapter")
        }
    }
    let engine = Engine::new(|_: &Cancellation| Ok(Panics), || {}).unwrap();
    for _ in 0..2 {
        engine.start(ClickConfig::default()).unwrap();
        wait_until(|| engine.snapshot().state == RunState::Faulted);
        assert!(engine.snapshot().error.unwrap().contains("unexpectedly"));
        let metrics = engine.snapshot().metrics;
        assert_eq!(metrics.attempts, 1);
        assert_eq!(metrics.lateness_p99, metrics.max_lateness);
    }
}

#[test]
fn first_pair_is_visible_while_waiting_for_a_long_interval() {
    let count = Arc::new(AtomicU64::new(0));
    let shared = count.clone();
    let engine = Engine::new(move |_: &Cancellation| Ok(Counter(shared.clone())), || {}).unwrap();
    engine
        .start(ClickConfig::new(999_999, MouseButton::Left, None).unwrap())
        .unwrap();
    wait_until(|| engine.snapshot().metrics.sent == 1);
    assert_eq!(count.load(Ordering::Relaxed), 1);
}

#[test]
fn manual_delay_begins_after_backend_preparation_and_remains_cancelable() {
    let count = Arc::new(AtomicU64::new(0));
    let shared = count.clone();
    let (release, released) = std::sync::mpsc::sync_channel(1);
    let engine = Engine::new(
        move |cancel: &Cancellation| {
            while released.try_recv().is_err() {
                if cancel.is_cancelled() {
                    return Err(BackendError::new("Cancelled"));
                }
                thread::sleep(Duration::from_millis(1));
            }
            Ok(Counter(shared.clone()))
        },
        || {},
    )
    .unwrap();
    engine
        .start_with_delay(ClickConfig::default(), Duration::from_secs(3))
        .unwrap();
    assert!(engine.snapshot().ready_at.is_none());
    let permission_time = Instant::now();
    release.send(()).unwrap();
    wait_until(|| engine.snapshot().ready_at.is_some());
    assert!(engine.snapshot().ready_at.unwrap() >= permission_time + Duration::from_secs(3));
    engine.stop();
    wait_until(|| engine.snapshot().state == RunState::Idle);
    assert_eq!(count.load(Ordering::Relaxed), 0);
}

#[test]
fn stopping_a_prepared_delay_finishes_native_resources_before_ack() {
    use std::sync::atomic::AtomicBool;
    struct Prepared(Arc<AtomicBool>);
    impl InputSink for Prepared {
        fn click(&mut self, _: MouseButton) -> Result<(), BackendError> {
            panic!("delay was bypassed")
        }
        fn finish(&mut self) -> Result<(), BackendError> {
            self.0.store(true, Ordering::Release);
            Ok(())
        }
    }
    let finished = Arc::new(AtomicBool::new(false));
    let shared = finished.clone();
    let engine = Engine::new(move |_: &Cancellation| Ok(Prepared(shared.clone())), || {}).unwrap();
    engine
        .start_with_delay(ClickConfig::default(), Duration::from_secs(3))
        .unwrap();
    wait_until(|| engine.snapshot().ready_at.is_some());
    engine.stop();
    wait_until(|| engine.snapshot().state == RunState::Idle);
    assert!(finished.load(Ordering::Acquire));
    assert!(engine.snapshot().ready_at.is_none());
}

#[test]
fn rejects_unbounded_preparation_delay_without_starting() {
    let count = Arc::new(AtomicU64::new(0));
    let shared = count.clone();
    let engine = Engine::new(move |_: &Cancellation| Ok(Counter(shared.clone())), || {}).unwrap();
    assert!(
        engine
            .start_with_delay(ClickConfig::default(), Duration::MAX)
            .is_err()
    );
    assert_eq!(engine.snapshot().state, RunState::Idle);
    assert_eq!(count.load(Ordering::Relaxed), 0);
}
