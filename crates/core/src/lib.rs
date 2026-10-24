//! Validated click plans and the desktop-independent execution engine.
#![forbid(unsafe_code)]
mod config;
mod scheduler;
pub use config::{ClickConfig, ConfigError, MAX_VALUE, MouseButton, StopCondition};
pub use scheduler::{ScheduleOverflow, Scheduler};
mod engine;
mod histogram;
mod ports;
mod state;
mod waiter;
pub use engine::{Engine, EngineError};
pub use ports::{BackendError, Cancellation, InputSink, Waiter};
pub use state::{Metrics, RunState, Snapshot, StopReason};

pub use waiter::StdWaiter;
