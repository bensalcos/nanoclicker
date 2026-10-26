mod controls;
use crate::{
    controller::Controller,
    diagnostics::Diagnostics,
    hotkey::Shortcut,
    settings::{Preferences, PreferencesFile},
    theme,
};
use eframe::egui::{self, Color32, RichText};
use nanoclick_core::{Engine, MouseButton, RunState, Snapshot, StopReason};
use nanoclick_platform::{Desktop, NativeInput};
use std::{
    sync::mpsc::{self, Receiver},
    time::Duration,
};

pub struct NanoClick {
    diagnostics: Option<Diagnostics>,
    diagnostics_message: Option<String>,
    preferences: Option<PreferencesFile>,
    preferences_message: Option<String>,
    preferences_recovery_available: bool,
    control: Controller,
    shortcut: Shortcut,
    desktop: Desktop,
    probe: Receiver<Result<(), String>>,
    available: Option<Result<(), String>>,
    theme: egui::ThemePreference,
    #[cfg(target_os = "windows")]
    window_fit: Option<([i32; 4], f32)>,
}
impl NanoClick {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        selected_theme: Option<egui::ThemePreference>,
        diagnostics: Option<Diagnostics>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ctx = cc.egui_ctx.clone();
        theme::configure(&ctx);
        let (preferences, saved, preferences_message, preferences_recovery_available) =
            match PreferencesFile::discover() {
                Ok(mut file) => match file.load() {
                    Ok(saved) => (Some(file), saved, None, false),
                    Err(error) => {
                        let recovery_available = file.can_recover();
                        (
                            Some(file),
                            Preferences::default(),
                            Some(error.to_string()),
                            recovery_available,
                        )
                    }
                },
                Err(error) => (None, Preferences::default(), Some(error.to_string()), false),
            };
        let selected_theme = selected_theme.unwrap_or_else(|| saved.theme.into());
        ctx.set_theme(selected_theme);
        let desktop = Desktop::detect();
        #[cfg(target_os = "linux")]
        let portal = {
            use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
            if desktop == Desktop::Wayland {
                match (cc.window_handle(), cc.display_handle()) {
                    (Ok(window), Ok(display)) => {
                        nanoclick_platform::wayland::PortalContext::from_window(
                            window.as_raw(),
                            display.as_raw(),
                        )
                    }
                    _ => Default::default(),
                }
            } else {
                Default::default()
            }
        };
        #[cfg(target_os = "linux")]
        let input_portal = portal.clone();
        let wake = ctx.clone();
        let engine = Engine::with_waiter(
            move |cancel| {
                #[cfg(target_os = "linux")]
                if desktop == Desktop::Wayland {
                    return input_portal
                        .open(cancel)
                        .map(|input| NativeInput::Wayland(Box::new(input)));
                }
                NativeInput::open(desktop, cancel)
            },
            nanoclick_platform::waiter()?,
            move || wake.request_repaint(),
        )?;
        let (sender, probe) = mpsc::sync_channel(1);
        let wake = ctx.clone();
        std::thread::Builder::new()
            .name("nanoclick-probe".into())
            .spawn(move || {
                let _ = sender.send(desktop.probe().map_err(|e| e.to_string()));
                wake.request_repaint();
            })?;
        let mut control = Controller::new(engine);
        control.draft = saved.draft();
        let shortcut = Shortcut::new(desktop, &ctx, saved.shortcut);
        #[cfg(target_os = "linux")]
        let shortcut = {
            let mut shortcut = shortcut;
            shortcut.portal_context = portal;
            shortcut
        };
        Ok(Self {
            diagnostics,
            diagnostics_message: None,
            preferences,
            preferences_message,
            preferences_recovery_available,
            control,
            shortcut,
            desktop,
            probe,
            available: None,
            theme: selected_theme,
            #[cfg(target_os = "windows")]
            window_fit: None,
        })
    }
    fn observe_diagnostics(&mut self, snapshot: &Snapshot) {
        if let Some(recorder) = self.diagnostics.as_mut()
            && let Err(error) = recorder.observe(snapshot, self.desktop)
        {
            self.diagnostics_message = Some(format!("Diagnostics export stopped: {error}"));
            self.diagnostics = None;
        }
    }
    fn save_preferences(&mut self) {
        let saved =
            Preferences::from_draft(&self.control.draft, self.theme, &self.shortcut.requested)
                .and_then(|value| match &self.preferences {
                    Some(file) => file.save(&value),
                    None => Err(crate::settings::SettingsError::Invalid(
                        "No preferences directory is available.".into(),
                    )),
                });
        self.preferences_message = Some(match saved {
            Ok(()) => "Preferences saved.".into(),
            Err(error) => error.to_string(),
        });
    }
    fn recover_preferences(&mut self) {
        let recovered =
            Preferences::from_draft(&self.control.draft, self.theme, &self.shortcut.requested)
                .and_then(|value| match &mut self.preferences {
                    Some(file) => file
                        .recover_and_save(&value)
                        .map(|backup| backup.display().to_string()),
                    None => Err(crate::settings::SettingsError::Invalid(
                        "No preferences directory is available.".into(),
                    )),
                });
        match recovered {
            Ok(backup) => {
                self.preferences_recovery_available = false;
                self.preferences_message = Some(format!(
                    "Current settings saved. The original file was backed up to {backup}."
                ));
            }
            Err(error) => {
                let backup_exists = self
                    .preferences
                    .as_ref()
                    .is_some_and(PreferencesFile::can_replace);
                self.preferences_recovery_available = self
                    .preferences
                    .as_ref()
                    .is_some_and(PreferencesFile::can_recover);
                self.preferences_message = Some(if backup_exists {
                    format!(
                        "The original file is backed up, but saving failed: {error}. Use Save to retry."
                    )
                } else {
                    error.to_string()
                });
            }
        }
    }
    fn controls(&mut self, ui: &mut egui::Ui, active: bool) {
        ui.add_enabled_ui(!active, |ui| {
            controls::plan(ui, &mut self.control.draft);
            section(ui, "03", "Global shortcut", |ui| {
                if self.desktop == Desktop::Wayland {
                    if ui.button("Configure in desktop…").clicked() {
                        self.shortcut.configure_portal(ui.ctx());
                    }
                    ui.weak("The desktop assigns the shortcut. Manual control is also available.");
                } else {
                    if controls::shortcut(ui, &mut self.shortcut.requested) {
                        self.shortcut.apply();
                    }
                }
                ui.small(&self.shortcut.status);
                if let Some(error) = &self.shortcut.error {
                    ui.colored_label(error_color(ui), error);
                }
            });
        });
    }
}
impl eframe::App for NanoClick {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "windows")]
        fit_window_to_monitor(ctx, _frame, &mut self.window_fit);

        if ctx.input(|input| {
            input.key_pressed(egui::Key::Escape) || input.viewport().close_requested()
        }) {
            self.stop();
            return;
        }
        if let Ok(result) = self.probe.try_recv() {
            self.available = Some(result);
        }
        self.process_shortcuts(ctx);
        self.shortcut.refresh();
        if self.control.engine.snapshot().ready_at.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.show(ui);
    }
}
impl NanoClick {
    fn process_shortcuts(&mut self, ctx: &egui::Context) {
        if !matches!(self.available, Some(Ok(()))) {
            self.shortcut.clear_pending();
            return;
        }
        let state = self.control.engine.snapshot().state;
        // Keep a later Start queued until the preceding Stop is acknowledged.
        // The engine's terminal wake resumes handling; never poll while stopping.
        if state == RunState::Stopping {
            return;
        }
        if self.shortcut.take_toggle() {
            self.control.toggle(false);
        }
        if self.shortcut.has_pending() {
            ctx.request_repaint();
        }
    }
    fn stop(&mut self) {
        self.shortcut.clear_pending();
        self.control.stop();
    }
    fn show(&mut self, ui: &mut egui::Ui) {
        self.show_snapshot(ui, &self.control.engine.snapshot());
    }
    fn show_snapshot(&mut self, ui: &mut egui::Ui, snapshot: &Snapshot) {
        self.observe_diagnostics(snapshot);
        let (label, status) = session_action(snapshot);
        let progress = format!(
            "{} pairs sent · {} periods skipped",
            snapshot.metrics.sent, snapshot.metrics.skipped
        );
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(24))
            .show(ui, |ui| {
                self.header(ui);
                ui.weak(self.desktop.name());
                ui.add_space(4.0);
                // Reserve the measured footer before drawing the scroll area,
                // but create its interactive widgets last: Tab follows reading order.
                let footer_height = action_height(ui, &status, &progress);
                let body_height = (ui.available_height() - footer_height - ui.spacing().item_spacing.y).max(0.0);
                egui::ScrollArea::vertical()
                    .max_height(body_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match &self.available {
                            None => { ui.weak("Checking desktop capabilities…"); }
                            Some(Err(error)) => { ui.colored_label(error_color(ui), error); }
                            _ => {}
                        }
                        if let Some(message) = &self.preferences_message { ui.label(message); }
                        if self.preferences_recovery_available {
                            ui.weak("NanoClick is using safe defaults. You can preserve the unreadable file and save the current settings.");
                            if ui
                                .button("Back up and replace damaged preferences")
                                .on_hover_text("Creates a unique backup beside the original, then saves the current settings.")
                                .clicked()
                            {
                                self.recover_preferences();
                            }
                        }
                        if let Some(message) = &self.diagnostics_message { ui.colored_label(error_color(ui), message); }
                        if let Some(error) = self.control.error.as_ref().or(snapshot.error.as_ref()) {
                            ui.colored_label(error_color(ui), error);
                        }
                        self.controls(ui, snapshot.state.is_active());
                        ui.add_space(8.0);
                        ui.small("Clicks are sent at the current pointer location. Escape stops while this window is focused.");
                    });
                self.action(ui, snapshot, label, &status, &progress);
            });
    }
}

#[cfg(target_os = "windows")]
fn fit_window_to_monitor(
    ctx: &egui::Context,
    frame: &eframe::Frame,
    last_fit: &mut Option<([i32; 4], f32)>,
) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let Ok(handle) = frame.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let Some(work) = nanoclick_platform::windows_window::work_area(handle.hwnd) else {
        return;
    };
    let (inner, outer, scale, maximized, fullscreen) = ctx.input(|input| {
        let viewport = input.viewport();
        (
            viewport.inner_rect,
            viewport.outer_rect,
            viewport.native_pixels_per_point,
            viewport.maximized,
            viewport.fullscreen,
        )
    });
    let (Some(inner), Some(outer), Some(scale)) = (inner, outer, scale) else {
        return;
    };
    if !scale.is_finite() || scale <= 0.0 || maximized == Some(true) || fullscreen == Some(true) {
        return;
    }
    let signature = (work, scale);
    if last_fit.as_ref() == Some(&signature) {
        return;
    }
    *last_fit = Some(signature);
    let work = egui::Rect::from_min_max(
        egui::pos2(work[0] as f32 / scale, work[1] as f32 / scale),
        egui::pos2(work[2] as f32 / scale, work[3] as f32 / scale),
    );
    let (size, position) = constrained_window_geometry(work, inner.size(), outer);
    let available = (work.size() - (outer.size() - inner.size())).max(egui::Vec2::splat(1.0));
    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(
        420.0_f32.min(available.x),
        520.0_f32.min(available.y),
    )));
    if let Some(size) = size {
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
    }
    if let Some(position) = position {
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(position));
    }
}

#[cfg(any(target_os = "windows", test))]
fn constrained_window_geometry(
    work: egui::Rect,
    inner_size: egui::Vec2,
    outer: egui::Rect,
) -> (Option<egui::Vec2>, Option<egui::Pos2>) {
    if !work.is_finite()
        || !outer.is_finite()
        || !inner_size.is_finite()
        || work.width() <= 0.0
        || work.height() <= 0.0
        || inner_size.x <= 0.0
        || inner_size.y <= 0.0
    {
        return (None, None);
    }
    let chrome = (outer.size() - inner_size).max(egui::Vec2::ZERO);
    let available = (work.size() - chrome).max(egui::Vec2::splat(1.0));
    let fitted = inner_size.min(available);
    let size = ((fitted - inner_size).length_sq() > 1.0).then_some(fitted);
    let max = (work.max - fitted - chrome).max(work.min);
    let fitted_position = egui::pos2(
        outer.min.x.clamp(work.min.x, max.x),
        outer.min.y.clamp(work.min.y, max.y),
    );
    let position = ((fitted_position - outer.min).length_sq() > 1.0).then_some(fitted_position);
    (size, position)
}

#[cfg(test)]
mod window_size_tests {
    use super::constrained_window_geometry;
    use eframe::egui::{Rect, pos2, vec2};
    #[test]
    fn preserves_window_on_secondary_monitor_with_positive_or_negative_origin() {
        for x in [1920.0, -1920.0] {
            let work = Rect::from_min_size(pos2(x, 0.0), vec2(1920.0, 1040.0));
            let outer = Rect::from_min_size(pos2(x + 100.0, 80.0), vec2(550.0, 630.0));
            assert_eq!(
                constrained_window_geometry(work, vec2(520.0, 600.0), outer),
                (None, None)
            );
        }
    }
    #[test]
    fn respects_top_and_left_taskbar_work_area() {
        let work = Rect::from_min_max(pos2(-1840.0, 60.0), pos2(0.0, 1080.0));
        let outer = Rect::from_min_size(pos2(-1900.0, 0.0), vec2(550.0, 630.0));
        assert_eq!(
            constrained_window_geometry(work, vec2(520.0, 600.0), outer),
            (None, Some(work.min))
        );
    }
    #[test]
    fn shrinks_to_actual_work_area_and_accounts_for_window_chrome() {
        let work = Rect::from_min_size(pos2(1280.0, 40.0), vec2(1024.0, 560.0));
        let outer = Rect::from_min_size(pos2(1360.0, 80.0), vec2(550.0, 630.0));
        assert_eq!(
            constrained_window_geometry(work, vec2(520.0, 600.0), outer),
            (Some(vec2(520.0, 530.0)), Some(pos2(1360.0, 40.0)))
        );
    }
}
impl NanoClick {
    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("NanoClick");
            let save_width = egui::WidgetText::from("Save")
                .into_galley(ui, None, f32::INFINITY, egui::TextStyle::Button)
                .size()
                .x
                + 2.0 * ui.spacing().button_padding.x;
            let actions_width = 100.0 + ui.spacing().item_spacing.x + save_width;
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(actions_width, ui.spacing().interact_size.y),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        controls::theme(ui, &mut self.theme);
                        if ui
                            .button("Save")
                            .on_hover_text("Save preferences for the next launch")
                            .clicked()
                        {
                            self.save_preferences();
                        }
                    },
                );
            });
        });
    }
    fn action(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &Snapshot,
        label: &str,
        status: &str,
        progress: &str,
    ) {
        ui.painter().hline(
            ui.max_rect().x_range(),
            ui.cursor().min.y,
            ui.visuals().widgets.noninteractive.bg_stroke,
        );
        egui::Frame::NONE
            .inner_margin(egui::Margin {
                top: 12,
                bottom: 0,
                left: 0,
                right: 0,
            })
            .show(ui, |ui| {
                let status_response =
                    ui.add(egui::Label::new(RichText::new(status).strong()).wrap());
                ui.ctx().accesskit_node_builder(status_response.id, |node| {
                    node.set_role(egui::Role::Status);
                    node.set_live(egui::accesskit::Live::Polite);
                    node.set_live_atomic();
                });
                ui.add(egui::Label::new(RichText::new(progress).weak()).wrap());
                let active = snapshot.state.is_active();
                let enabled = snapshot.state != RunState::Stopping
                    && (active || matches!(self.available, Some(Ok(()))));
                let button = egui::Button::new(
                    RichText::new(label).size(17.0).color(Color32::WHITE),
                )
                .fill(if active {
                    theme::RED
                } else {
                    Color32::from_rgb(30, 100, 200)
                });
                if ui
                    .add_enabled(
                        enabled,
                        button.min_size(egui::vec2(ui.available_width(), 46.0)),
                    )
                    .clicked()
                {
                    self.shortcut.clear_pending();
                    self.control.toggle(true);
                }
            });
    }
}
fn session_action(snapshot: &Snapshot) -> (&'static str, String) {
    if let Some(deadline) = snapshot.ready_at
        && snapshot.state == RunState::Preparing
    {
        return (
            "Cancel",
            format!(
                "Starting in {} · move the pointer to your target",
                deadline
                    .saturating_duration_since(std::time::Instant::now())
                    .as_secs()
                    + 1
            ),
        );
    }
    match snapshot.state {
        RunState::Preparing => ("Cancel", "Preparing desktop input…".to_owned()),
        RunState::Running => ("Stop", "Running".to_owned()),
        RunState::Stopping => ("Stopping…", "Releasing input…".to_owned()),
        RunState::Faulted => ("Start", "Could not complete the session".to_owned()),
        RunState::Idle => (
            "Start",
            if snapshot.reason == Some(StopReason::Completed) {
                "Completed"
            } else {
                "Ready"
            }
            .to_owned(),
        ),
    }
}
fn action_height(ui: &egui::Ui, status: &str, progress: &str) -> f32 {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let height = |text: &str| {
        ui.painter()
            .layout(
                text.to_owned(),
                font.clone(),
                ui.visuals().text_color(),
                ui.available_width(),
            )
            .size()
            .y
    };
    12.0 + height(status) + height(progress) + 2.0 * ui.spacing().item_spacing.y + 46.0
}

fn section(ui: &mut egui::Ui, number: &str, title: &str, contents: impl FnOnce(&mut egui::Ui)) {
    ui.label(
        RichText::new(format!("{number}   {title}"))
            .strong()
            .size(15.0),
    );
    egui::Frame::group(ui.style())
        .inner_margin(8)
        .corner_radius(10)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            contents(ui);
        });
}
fn button_name(button: MouseButton) -> &'static str {
    match button {
        MouseButton::Left => "Left",
        MouseButton::Right => "Right",
        MouseButton::Middle => "Middle",
    }
}
fn error_color(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(255, 140, 150)
    } else {
        Color32::from_rgb(170, 25, 40)
    }
}

#[cfg(test)]
mod tests;
