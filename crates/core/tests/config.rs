#![expect(
    clippy::unwrap_used,
    reason = "Test fixtures fail immediately when their preconditions are not met."
)]
use nanoclick_core::{ClickConfig, MouseButton, StopCondition};

#[test]
fn rejects_interval_outside_supported_range() {
    for value in [0, 1_000_000, u64::MAX] {
        assert!(ClickConfig::new(value, MouseButton::Left, None).is_err());
    }
}

#[test]
fn rejects_zero_and_oversized_counts() {
    for value in [0, 1_000_000, u64::MAX] {
        assert!(ClickConfig::new(100, MouseButton::Left, Some(value)).is_err());
    }
}

#[test]
fn preserves_valid_plan_and_defaults() {
    let plan = ClickConfig::new(1, MouseButton::Middle, Some(999_999)).unwrap();
    assert_eq!(plan.interval().as_millis(), 1);
    assert_eq!(plan.button(), MouseButton::Middle);
    assert!(matches!(plan.stop_condition(), StopCondition::Count(n) if n.get() == 999_999));
    assert_eq!(ClickConfig::default().interval().as_millis(), 100);
    assert_eq!(
        ClickConfig::default().stop_condition(),
        StopCondition::UntilStopped
    );
}
