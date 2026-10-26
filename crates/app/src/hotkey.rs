use eframe::egui;
use global_hotkey::hotkey::HotKey;
use nanoclick_platform::Desktop;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

type RegistrationUpdate = Result<String, String>;
pub struct Shortcut {
    desktop: Desktop,
    ctx: egui::Context,
    #[cfg(target_os = "linux")]
    x11: Option<nanoclick_platform::x11_shortcut::X11Shortcut>,
    #[cfg(target_os = "linux")]
    portal: Option<nanoclick_platform::portal_shortcut::PortalShortcut>,
    #[cfg(target_os = "linux")]
    pub portal_context: nanoclick_platform::wayland::PortalContext,
    #[cfg(target_os = "windows")]
    windows: Option<nanoclick_platform::windows_shortcut::WindowsShortcut>,
    latest: Arc<Mutex<Option<RegistrationUpdate>>>,
    current: Option<HotKey>,
    pub requested: String,
    pub status: String,
    pub error: Option<String>,
    events: Arc<AtomicUsize>,
}
impl Shortcut {
    pub fn new(desktop: Desktop, ctx: &egui::Context, requested: String) -> Self {
        let mut shortcut = Self::unregistered(desktop, ctx, requested);
        if desktop == Desktop::Wayland {
            shortcut.status = "Not configured · use the desktop portal".into();
        } else {
            shortcut.apply();
        }
        shortcut
    }
    /// Creates state without claiming any native registration or opening dialogs.
    pub(crate) fn unregistered(desktop: Desktop, ctx: &egui::Context, requested: String) -> Self {
        Self {
            desktop,
            ctx: ctx.clone(),
            #[cfg(target_os = "linux")]
            x11: None,
            #[cfg(target_os = "linux")]
            portal: None,
            #[cfg(target_os = "linux")]
            portal_context: Default::default(),
            #[cfg(target_os = "windows")]
            windows: None,
            latest: Arc::new(Mutex::new(None)),
            current: None,
            requested,
            status: "Not registered".into(),
            error: None,
            events: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub(crate) fn activation(&self) -> impl Fn() + Send + Sync + 'static + use<> {
        let events = self.events.clone();
        let ctx = self.ctx.clone();
        move || {
            events.fetch_add(1, Ordering::Release);
            ctx.request_repaint();
        }
    }
    fn reporter(
        &self,
        latest: Arc<Mutex<Option<RegistrationUpdate>>>,
    ) -> impl Fn(RegistrationUpdate) + Send + Sync + 'static + use<> {
        let ctx = self.ctx.clone();
        move |update| {
            *latest
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(update);
            ctx.request_repaint();
        }
    }
    pub fn configure_portal(&mut self, _: &egui::Context) {
        #[cfg(target_os = "linux")]
        {
            self.portal = None;
            self.latest = Arc::new(Mutex::new(None));
            self.events.store(0, Ordering::Release);
            self.status = "Waiting for desktop shortcut configuration…".into();
            self.error = None;
            let report = self.reporter(self.latest.clone());
            match nanoclick_platform::portal_shortcut::PortalShortcut::configure_with_context(
                self.portal_context.clone(),
                move |update| {
                    use nanoclick_platform::portal_shortcut::ShortcutUpdate;
                    report(match update {
                        ShortcutUpdate::Registered(trigger) => Ok(trigger),
                        ShortcutUpdate::Unavailable(error) => Err(error),
                    });
                },
                self.activation(),
            ) {
                Ok(portal) => self.portal = Some(portal),
                Err(error) => self.error = Some(error.to_string()),
            }
        }
    }
    pub fn refresh(&mut self) {
        let update = self
            .latest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(update) = update {
            match update {
                Ok(trigger) => {
                    self.status = format!("Registered · {trigger}");
                    self.error = None;
                }
                Err(error) => {
                    self.current = None;
                    #[cfg(target_os = "linux")]
                    {
                        self.x11 = None;
                    }
                    #[cfg(target_os = "windows")]
                    {
                        self.windows = None;
                    }
                    self.status = "Not registered · manual control available".into();
                    self.error = Some(error);
                }
            }
        }
    }
    pub fn apply(&mut self) {
        if self.desktop == Desktop::Wayland {
            return;
        }
        let parsed = match self.requested.parse::<HotKey>() {
            Ok(value) => value,
            Err(error) => {
                self.error = Some(error.to_string());
                return;
            }
        };
        if self.current == Some(parsed) {
            return;
        }
        let updates = Arc::new(Mutex::new(None));
        let report = self.reporter(updates.clone());
        let failed = move |error| report(Err(error));
        #[cfg(target_os = "linux")]
        let registration = nanoclick_platform::x11_shortcut::X11Shortcut::register(
            parsed,
            self.activation(),
            failed,
        );
        #[cfg(target_os = "windows")]
        let registration = nanoclick_platform::windows_shortcut::WindowsShortcut::register(
            parsed,
            self.activation(),
            failed,
        );
        match registration {
            Ok(listener) => {
                #[cfg(target_os = "linux")]
                {
                    self.x11 = Some(listener);
                }
                #[cfg(target_os = "windows")]
                {
                    self.windows = Some(listener);
                }
                self.latest = updates;
                self.current = Some(parsed);
                self.status = format!("Registered · {}", parsed.into_string());
                self.error = None;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub fn take_toggle(&self) -> bool {
        self.events
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |pending| {
                pending.checked_sub(1)
            })
            .is_ok()
    }
    pub fn has_pending(&self) -> bool {
        self.events.load(Ordering::Acquire) != 0
    }
    pub fn clear_pending(&self) {
        self.events.store(0, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_queued_shortcut_presses_are_delivered_once_each() {
        let ctx = egui::Context::default();
        let shortcut = Shortcut::unregistered(Desktop::X11, &ctx, "F6".into());
        let activate = shortcut.activation();
        activate();
        activate();
        assert!(
            shortcut.take_toggle(),
            "The first activation must survive a batch"
        );
        assert!(
            shortcut.take_toggle(),
            "The second activation must remain queued"
        );
        assert!(
            !shortcut.take_toggle(),
            "A consumed activation must not be replayed"
        );
    }
}
