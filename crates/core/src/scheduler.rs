use std::time::Duration;

/// Arithmetic-only scheduler whose epoch is supplied by its caller.
#[derive(Debug)]
pub struct Scheduler {
    interval: Duration,
    deadline: Duration,
}

/// A deadline or skipped-slot counter cannot be represented.
#[derive(Debug, thiserror::Error)]
#[error("The execution exceeded the supported time range.")]
pub struct ScheduleOverflow;

impl Scheduler {
    /// Starts at zero. A zero interval is rejected by `advance`.
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            deadline: Duration::ZERO,
        }
    }
    /// Offset of the next click from the run epoch.
    pub fn deadline(&self) -> Duration {
        self.deadline
    }
    /// Moves to a strictly future deadline, reporting omitted slots.
    pub fn advance(&mut self, now: Duration) -> Result<u64, ScheduleOverflow> {
        let period = self.interval.as_nanos();
        if period == 0 {
            return Err(ScheduleOverflow);
        }
        let next = self
            .deadline
            .checked_add(self.interval)
            .ok_or(ScheduleOverflow)?;
        let skipped = if now >= next {
            (now - next).as_nanos() / period + 1
        } else {
            0
        };
        let nanos = next
            .as_nanos()
            .checked_add(period.checked_mul(skipped).ok_or(ScheduleOverflow)?)
            .ok_or(ScheduleOverflow)?;
        let secs = u64::try_from(nanos / 1_000_000_000).map_err(|_| ScheduleOverflow)?;
        self.deadline = Duration::new(secs, (nanos % 1_000_000_000) as u32);
        u64::try_from(skipped).map_err(|_| ScheduleOverflow)
    }
}
