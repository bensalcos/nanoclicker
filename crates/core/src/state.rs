use std::time::{Duration, Instant};

/// Observable execution lifecycle.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    #[default]
    Idle,
    Preparing,
    Running,
    Stopping,
    Faulted,
}
impl RunState {
    /// Whether a plan is being prepared, executed, or stopped.
    pub fn is_active(self) -> bool {
        matches!(self, Self::Preparing | Self::Running | Self::Stopping)
    }
}

/// Why the last run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    Completed,
    Stopped,
    Failed,
}

/// Constant-space measurements, published at most ten times per second.
#[derive(Debug, Default, Clone, Copy)]
pub struct Metrics {
    pub attempts: u64,
    pub sent: u64,
    pub failures: u64,
    pub skipped: u64,
    pub max_lateness: Duration,
}

/// Latest authoritative state. Terminal state is retained until the next run.
#[derive(Debug, Default, Clone)]
pub struct Snapshot {
    pub run_id: u64,
    pub state: RunState,
    pub reason: Option<StopReason>,
    pub metrics: Metrics,
    /// Present only during the post-permission, cancelable manual delay.
    pub ready_at: Option<Instant>,
    pub error: Option<String>,
}
