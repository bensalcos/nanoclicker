#![expect(
    clippy::unwrap_used,
    reason = "Test fixtures fail immediately when their preconditions are not met."
)]
use nanoclick_core::Scheduler;
use std::time::Duration;

#[test]
fn first_deadline_is_immediate_and_subsequent_deadlines_do_not_drift() {
    let mut scheduler = Scheduler::new(Duration::from_millis(10));
    assert_eq!(scheduler.deadline(), Duration::ZERO);
    assert_eq!(scheduler.advance(Duration::from_millis(2)).unwrap(), 0);
    assert_eq!(scheduler.deadline(), Duration::from_millis(10));
    scheduler.advance(Duration::from_millis(12)).unwrap();
    assert_eq!(scheduler.deadline(), Duration::from_millis(20));
}

#[test]
fn skips_expired_slots_without_bursting() {
    let mut scheduler = Scheduler::new(Duration::from_millis(10));
    assert_eq!(scheduler.advance(Duration::from_millis(35)).unwrap(), 3);
    assert_eq!(scheduler.deadline(), Duration::from_millis(40));
}

#[test]
fn boundary_is_future_and_overflow_is_reported() {
    let mut scheduler = Scheduler::new(Duration::from_millis(10));
    assert_eq!(scheduler.advance(Duration::from_millis(10)).unwrap(), 1);
    assert_eq!(scheduler.deadline(), Duration::from_millis(20));
    assert!(scheduler.advance(Duration::MAX).is_err());
}
