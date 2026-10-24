use nanoclick_core::BackendError;
use std::time::Duration;

// Negative 100 ns units select a relative due time. Round up so conversion
// never intentionally schedules before the monotonic deadline, even below 100 ns.
pub(crate) fn relative_due(remaining: Duration) -> Result<i64, BackendError> {
    let ticks = remaining.as_nanos().div_ceil(100).max(1);
    i64::try_from(ticks)
        .map(|ticks| -ticks)
        .map_err(|_| BackendError::new("Windows timer deadline is out of range."))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rounds_relative_deadlines_up_and_rejects_overflow() -> Result<(), BackendError> {
        assert_eq!(relative_due(Duration::ZERO)?, -1);
        assert_eq!(relative_due(Duration::from_nanos(1))?, -1);
        assert_eq!(relative_due(Duration::from_nanos(100))?, -1);
        assert_eq!(relative_due(Duration::from_nanos(101))?, -2);
        assert_eq!(relative_due(Duration::from_millis(1))?, -10_000);
        assert!(relative_due(Duration::MAX).is_err());
        Ok(())
    }
}
