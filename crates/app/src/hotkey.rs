use eframe::egui;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::HotKey};
use nanoclick_platform::Desktop;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
};

pub struct Shortcut {
    #[cfg(target_os = "linux")]
    ctx: egui::Context,
    #[cfg(target_os = "linux")]
    desktop: Desktop,
    #[cfg(target_os = "linux")]
    x11: Option<nanoclick_platform::x11_shortcut::X11Shortcut>,
    #[cfg(target_os = "linux")]
    portal: Option<nanoclick_platform::portal_shortcut::PortalShortcut>,
    #[cfg(target_os = "linux")]
    portal_update:
        Arc<std::sync::Mutex<Option<nanoclick_platform::portal_shortcut::ShortcutUpdate>>>,
    manager: Option<GlobalHotKeyManager>,
    current: Option<HotKey>,
    pub requested: String,
    pub status: String,
    pub error: Option<String>,
    events: Arc<AtomicUsize>,
    active_id: Arc<AtomicU32>,
}
impl Shortcut {
    pub fn new(desktop: Desktop, ctx: &egui::Context, requested: String) -> Self {
        let events = Arc::new(AtomicUsize::new(0));
        let active_id = Arc::new(AtomicU32::new(0));
        let mut shortcut = Self {
            #[cfg(target_os = "linux")]
            ctx: ctx.clone(),
            #[cfg(target_os = "linux")]
            desktop,
            #[cfg(target_os = "linux")]
            x11: None,
            #[cfg(target_os = "linux")]
            portal: None,
            #[cfg(target_os = "linux")]
            portal_update: Arc::new(std::sync::Mutex::new(None)),
            manager: None,
            current: None,
            requested,
            status: "Not registered".into(),
            error: None,
            events: events.clone(),
            active_id: active_id.clone(),
        };
        if desktop == Desktop::Wayland {
            shortcut.status = "Not configured · use the desktop portal".into();
            return shortcut;
        }
        #[cfg(target_os = "linux")]
        if desktop == Desktop::X11 {
            shortcut.apply();
            return shortcut;
        }
        match GlobalHotKeyManager::new() {
            Ok(manager) => shortcut.manager = Some(manager),
            Err(error) => {
                shortcut.error = Some(error.to_string());
                return shortcut;
            }
        }
        let ctx = ctx.clone();
        let held = AtomicBool::new(false);
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.id != active_id.load(Ordering::Acquire) {
                return;
            }
            match event.state {
                HotKeyState::Pressed if !held.swap(true, Ordering::AcqRel) => {
                    events.fetch_add(1, Ordering::Release);
                    ctx.request_repaint();
                }
                HotKeyState::Released => {
                    held.store(false, Ordering::Release);
                }
                _ => {}
            }
        }));
        shortcut.apply();
        shortcut
    }
    pub fn configure_portal(&mut self, ctx: &egui::Context) {
        #[cfg(target_os = "linux")]
        {
            self.portal = None;
            *self
                .portal_update
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            self.events.store(0, Ordering::Release);
            let latest = self.portal_update.clone();
            let changed = ctx.clone();
            let triggered = ctx.clone();
            let events = self.events.clone();
            self.status = "Waiting for desktop shortcut configuration…".into();
            self.error = None;
            match nanoclick_platform::portal_shortcut::PortalShortcut::configure(
                move |update| {
                    *latest
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(update);
                    changed.request_repaint();
                },
                move || {
                    events.fetch_add(1, Ordering::Release);
                    triggered.request_repaint();
                },
            ) {
                Ok(portal) => self.portal = Some(portal),
                Err(error) => self.error = Some(error.to_string()),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = ctx;
        }
    }
    pub fn refresh(&mut self) {
        #[cfg(target_os = "linux")]
        if let Some(update) = self
            .portal_update
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            use nanoclick_platform::portal_shortcut::ShortcutUpdate;
            match update {
                ShortcutUpdate::Registered(trigger) => {
                    self.status = format!("Registered · {trigger}");
                    self.error = None;
                }
                ShortcutUpdate::Unavailable(error) => {
                    self.current = None;
                    self.x11 = None;
                    self.status = "Not registered · manual control available".into();
                    self.error = Some(error);
                }
            }
        }
    }
    pub fn apply(&mut self) {
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
        #[cfg(target_os = "linux")]
        if self.desktop == Desktop::X11 {
            let events = self.events.clone();
            let changed = self.ctx.clone();
            let failed_ctx = self.ctx.clone();
            let update = self.portal_update.clone();
            match nanoclick_platform::x11_shortcut::X11Shortcut::register(
                parsed,
                move || {
                    events.fetch_add(1, Ordering::Release);
                    changed.request_repaint();
                },
                move |error| {
                    *update
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(
                        nanoclick_platform::portal_shortcut::ShortcutUpdate::Unavailable(error),
                    );
                    failed_ctx.request_repaint();
                },
            ) {
                Ok(listener) => {
                    self.x11 = Some(listener);
                    self.current = Some(parsed);
                    self.status = format!("Registered · {}", parsed.into_string());
                    self.error = None;
                }
                Err(error) => self.error = Some(error.to_string()),
            }
            return;
        }
        let Some(manager) = &self.manager else {
            return;
        };
        if let Err(error) = manager.register(parsed) {
            self.error = Some(format!("Shortcut unavailable: {error}"));
            return;
        }
        if let Some(old) = self.current
            && let Err(error) = manager.unregister(old)
        {
            let _ = manager.unregister(parsed);
            self.error = Some(format!("Could not replace shortcut: {error}"));
            return;
        }
        self.current = Some(parsed);
        self.active_id.store(parsed.id(), Ordering::Release);
        self.status = format!("Registered · {}", parsed.into_string());
        self.error = None;
    }
    pub fn take_toggle(&self) -> bool {
        self.events.swap(0, Ordering::AcqRel) % 2 == 1
    }
}
impl Drop for Shortcut {
    fn drop(&mut self) {
        self.active_id.store(0, Ordering::Release);
        if let (Some(manager), Some(key)) = (&self.manager, self.current) {
            let _ = manager.unregister(key);
        }
    }
}
