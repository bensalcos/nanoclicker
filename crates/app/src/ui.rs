use crate::{
    controller::Controller,
    hotkey::Shortcut,
    settings::{Preferences, PreferencesFile},
    theme,
};
use eframe::egui::{self, Color32, RichText};
use nanoclick_core::{Engine, MouseButton, RunState, StopReason};
use nanoclick_platform::{Desktop, NativeInput};
use std::{
    sync::mpsc::{self, Receiver},
    time::Duration,
};

pub struct NanoClick {
    preferences: Option<PreferencesFile>,
    preferences_message: Option<String>,
    control: Controller,
    shortcut: Shortcut,
    desktop: Desktop,
    probe: Receiver<Result<(), String>>,
    available: Option<Result<(), String>>,
    theme: egui::ThemePreference,
}
impl NanoClick {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        selected_theme: Option<egui::ThemePreference>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ctx = cc.egui_ctx.clone();
        theme::configure(&ctx);
        let (preferences, saved, preferences_message) = match PreferencesFile::discover() {
            Ok(mut file) => match file.load() {
                Ok(saved) => (Some(file), saved, None),
                Err(error) => (Some(file), Preferences::default(), Some(error.to_string())),
            },
            Err(error) => (None, Preferences::default(), Some(error.to_string())),
        };
        let selected_theme = selected_theme.unwrap_or_else(|| saved.theme.into());
        ctx.set_theme(selected_theme);
        let desktop = Desktop::detect();
        let wake = ctx.clone();
        let engine = Engine::new(
            move |cancel| NativeInput::open(desktop, cancel),
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
        Ok(Self {
            preferences,
            preferences_message,
            control,
            shortcut: Shortcut::new(desktop, &ctx, saved.shortcut),
            desktop,
            probe,
            available: None,
            theme: selected_theme,
        })
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
    fn controls(&mut self, ui: &mut egui::Ui, active: bool) {
        ui.add_enabled_ui(!active, |ui| {
            section(ui, "01", "Timing", |ui| {
                ui.horizontal(|ui| {
                    let label = ui.label("Interval");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.control.draft.interval)
                            .desired_width(104.0)
                            .char_limit(6),
                    )
                    .labelled_by(label.id);
                    ui.label("ms");
                    if let Ok(config) = self.control.draft.validate() {
                        ui.weak(format!(
                            "≈ {:.1} clicks/s",
                            1.0 / config.interval().as_secs_f64()
                        ));
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Mouse button");
                    egui::ComboBox::from_id_salt("mouse-button")
                        .selected_text(button_name(self.control.draft.button))
                        .show_ui(ui, |ui| {
                            for button in
                                [MouseButton::Left, MouseButton::Right, MouseButton::Middle]
                            {
                                ui.selectable_value(
                                    &mut self.control.draft.button,
                                    button,
                                    button_name(button),
                                );
                            }
                        });
                });
            });
            section(ui, "02", "Repeat", |ui| {
                ui.radio_value(&mut self.control.draft.finite, false, "Until stopped");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.control.draft.finite, true, "Count");
                    ui.add_enabled(
                        self.control.draft.finite,
                        egui::TextEdit::singleline(&mut self.control.draft.count)
                            .desired_width(104.0)
                            .hint_text("Clicks")
                            .char_limit(6),
                    );
                    ui.weak("clicks");
                });
            });
            section(ui, "03", "Global shortcut", |ui| {
                if self.desktop == Desktop::Wayland {
                    if ui.button("Configure in desktop…").clicked() {
                        self.shortcut.configure_portal(ui.ctx());
                    }
                    ui.weak("The desktop assigns the shortcut. Manual control is also available.");
                } else {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.shortcut.requested)
                                .desired_width(160.0)
                                .hint_text("Ctrl+Shift+F6"),
                        );
                        if ui.button("Apply").clicked() {
                            self.shortcut.apply();
                        }
                    });
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
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Ok(result) = self.probe.try_recv() {
            self.available = Some(result);
        }
        if self.shortcut.take_toggle() && matches!(self.available, Some(Ok(()))) {
            self.control.toggle(false);
        }
        self.shortcut.refresh();
        if self.control.engine.snapshot().ready_at.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.control.stop();
        }
        if ctx.input(|input| input.viewport().close_requested()) {
            self.control.stop();
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let snapshot = self.control.engine.snapshot();
        let active = snapshot.state.is_active();
        egui::CentralPanel::default().frame(egui::Frame::central_panel(ui.style()).inner_margin(24)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("NanoClick");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Save").on_hover_text("Save preferences for the next launch").clicked() { self.save_preferences(); }
                    egui::ComboBox::from_id_salt("theme").selected_text(match self.theme { egui::ThemePreference::Dark => "Dark", egui::ThemePreference::Light => "Light", _ => "System" }).show_ui(ui, |ui| {
                        for (value, label) in [(egui::ThemePreference::System, "System"), (egui::ThemePreference::Light, "Light"), (egui::ThemePreference::Dark, "Dark")] {
                            if ui.selectable_value(&mut self.theme, value, label).changed() { ui.ctx().set_theme(self.theme); }
                        }
                    });
                });
            });
            ui.weak(self.desktop.name());
            ui.add_space(4.0);
            egui::Panel::bottom("action").frame(egui::Frame::NONE.inner_margin(egui::Margin { top: 12, bottom: 0, left: 0, right: 0 })).show(ui, |ui| {
                let (label, status) = if let Some(deadline) = snapshot.ready_at && snapshot.state == RunState::Preparing {
                    ("Cancel", format!("Starting in {} · move the pointer to your target", deadline.saturating_duration_since(std::time::Instant::now()).as_secs() + 1))
                } else {
                    match snapshot.state {
                        RunState::Preparing => ("Cancel", "Preparing desktop input…".to_owned()),
                        RunState::Running => ("Stop", "Running".to_owned()),
                        RunState::Stopping => ("Stopping…", "Releasing input…".to_owned()),
                        RunState::Faulted => ("Start", "Could not complete the session".to_owned()),
                        RunState::Idle => ("Start", if snapshot.reason == Some(StopReason::Completed) { "Completed" } else { "Ready" }.to_owned()),
                    }
                };
                ui.horizontal(|ui| { ui.label(RichText::new(status).strong()); });
                ui.weak(format!("{} pairs sent · {} periods skipped", snapshot.metrics.sent, snapshot.metrics.skipped));
                let enabled = snapshot.state != RunState::Stopping && (active || matches!(self.available, Some(Ok(()))));
                let button = egui::Button::new(RichText::new(label).size(17.0).color(Color32::WHITE)).fill(if active { theme::RED } else { Color32::from_rgb(30, 100, 200) });
                if ui.add_enabled(enabled, button.min_size(egui::vec2(ui.available_width(), 46.0))).clicked() { self.control.toggle(true); }
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                match &self.available {
                    None => { ui.weak("Checking desktop capabilities…"); }
                    Some(Err(error)) => { ui.colored_label(error_color(ui), error); }
                    _ => {}
                }
                if let Some(message) = &self.preferences_message { ui.label(message); }
                if let Some(error) = self.control.error.as_ref().or(snapshot.error.as_ref()) { ui.colored_label(error_color(ui), error); }
                self.controls(ui, active);
                ui.add_space(8.0);
                ui.small("Clicks are sent at the current pointer location. Escape stops while this window is focused.");
            });
        });
    }
}
fn section(ui: &mut egui::Ui, number: &str, title: &str, contents: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(4.0);
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
