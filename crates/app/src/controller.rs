use nanoclick_core::{ClickConfig, Engine, MouseButton};
use std::time::{Duration, Instant};

pub struct Draft {
    pub interval: String,
    pub count: String,
    pub finite: bool,
    pub button: MouseButton,
}
impl Default for Draft {
    fn default() -> Self {
        Self {
            interval: "100".into(),
            count: "10".into(),
            finite: false,
            button: MouseButton::Left,
        }
    }
}
impl Draft {
    pub fn validate(&self) -> Result<ClickConfig, String> {
        let interval = self
            .interval
            .parse::<u64>()
            .map_err(|_| "Enter an interval from 1 to 999999 milliseconds.".to_owned())?;
        let count = if self.finite {
            Some(
                self.count
                    .parse::<u64>()
                    .map_err(|_| "Enter a count from 1 to 999999 clicks.".to_owned())?,
            )
        } else {
            None
        };
        ClickConfig::new(interval, self.button, count).map_err(|e| e.to_string())
    }
}

pub struct Controller {
    pub draft: Draft,
    pub engine: Engine,
    pub countdown: Option<(Instant, ClickConfig)>,
    pub error: Option<String>,
}
impl Controller {
    pub fn new(engine: Engine) -> Self {
        Self {
            draft: Draft::default(),
            engine,
            countdown: None,
            error: None,
        }
    }
    pub fn toggle(&mut self, manual: bool) {
        if self.countdown.take().is_some() {
            return;
        }
        if self.engine.snapshot().state.is_active() {
            self.engine.stop();
            return;
        }
        match self.draft.validate() {
            Err(error) => self.error = Some(error),
            Ok(plan) => {
                self.error = None;
                if manual {
                    self.countdown = Some((Instant::now() + Duration::from_secs(3), plan));
                } else if let Err(error) = self.engine.start(plan) {
                    self.error = Some(error.to_string());
                }
            }
        }
    }
    pub fn tick(&mut self) {
        if let Some((deadline, plan)) = self.countdown
            && Instant::now() >= deadline
        {
            self.countdown = None;
            if let Err(error) = self.engine.start(plan) {
                self.error = Some(error.to_string());
            }
        }
    }
    pub fn stop(&mut self) {
        self.countdown = None;
        self.engine.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nanoclick_core::{BackendError, InputSink};
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };
    struct Counter(Arc<AtomicU64>);
    impl InputSink for Counter {
        fn click(&mut self, _: MouseButton) -> Result<(), BackendError> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }
    #[test]
    fn draft_reports_invalid_values_and_ignores_unused_count() {
        let mut draft = Draft {
            interval: "0".into(),
            ..Default::default()
        };
        assert!(draft.validate().is_err());
        draft.interval = "100".into();
        draft.count = "bad".into();
        assert!(draft.validate().is_ok());
        draft.finite = true;
        assert!(draft.validate().is_err());
    }
    #[test]
    fn manual_start_is_delayed_and_can_be_cancelled_without_input()
    -> Result<(), Box<dyn std::error::Error>> {
        let count = Arc::new(AtomicU64::new(0));
        let shared = count.clone();
        let engine = Engine::new(move |_| Ok(Counter(shared.clone())), || {})?;
        let mut control = Controller::new(engine);
        control.toggle(true);
        assert!(control.countdown.is_some());
        assert_eq!(count.load(Ordering::Relaxed), 0);
        control.toggle(true);
        control.tick();
        assert!(control.countdown.is_none());
        assert_eq!(count.load(Ordering::Relaxed), 0);
        Ok(())
    }
}
