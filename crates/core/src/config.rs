use std::{num::NonZeroU64, time::Duration};

/// Largest interval and finite count supported by the first release.
pub const MAX_VALUE: u64 = 999_999;

/// Mouse button at the current pointer location.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MouseButton {
    #[default]
    Left,
    Right,
    Middle,
}

/// When an execution ends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StopCondition {
    #[default]
    UntilStopped,
    Count(NonZeroU64),
}

/// A validated, immutable execution plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickConfig {
    interval: Duration,
    button: MouseButton,
    stop: StopCondition,
}

/// Invalid user-supplied configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("Interval must be between 1 and 999999 milliseconds.")]
    Interval,
    #[error("Count must be between 1 and 999999 clicks.")]
    Count,
}

impl ClickConfig {
    /// Validates a plan; `None` means unlimited clicks.
    pub fn new(
        interval_ms: u64,
        button: MouseButton,
        count: Option<u64>,
    ) -> Result<Self, ConfigError> {
        if !(1..=MAX_VALUE).contains(&interval_ms) {
            return Err(ConfigError::Interval);
        }
        let stop = match count {
            None => StopCondition::UntilStopped,
            Some(value) if value <= MAX_VALUE => {
                StopCondition::Count(NonZeroU64::new(value).ok_or(ConfigError::Count)?)
            }
            Some(_) => return Err(ConfigError::Count),
        };
        Ok(Self {
            interval: Duration::from_millis(interval_ms),
            button,
            stop,
        })
    }

    /// Desired period between click starts.
    pub fn interval(self) -> Duration {
        self.interval
    }
    /// Button to inject.
    pub fn button(self) -> MouseButton {
        self.button
    }
    /// Finite or unlimited repetition.
    pub fn stop_condition(self) -> StopCondition {
        self.stop
    }
}

impl Default for ClickConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(100),
            button: MouseButton::Left,
            stop: StopCondition::UntilStopped,
        }
    }
}
