//! Passive native input capture for desktop QA. No input injection.
#![forbid(unsafe_code)]
#[path = "../src/renderer.rs"]
mod renderer;
#[path = "input_capture/session.rs"]
mod session;
use eframe::egui;
use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use std::{
    fs::File,
    io::{self, BufWriter},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
const USAGE: &str = "Usage: input_capture NEW_JSONL_PATH\nSpace starts/ends a session; Escape saves the active session and closes.\nOnly this window's mouse-button events are counted. No input is injected.";
struct CaptureWindow {
    capture: session::Capture,
    output: BufWriter<File>,
    backend: &'static str,
    started: Option<(Instant, f32, u64, u64)>,
    message: Option<String>,
    output_failed: Arc<AtomicBool>,
    pointer_position: Option<egui::Pos2>,
    pointer_move_events: u64,
    pointer_button_events: u64,
}
impl CaptureWindow {
    fn toggle(&mut self, scale: f32) {
        if self.capture.active() {
            self.finish(scale, "paused");
        } else if !self.output_failed.load(Ordering::Acquire) {
            match self.capture.start() {
                Ok(()) => {
                    self.started = Some((
                        Instant::now(),
                        scale,
                        self.pointer_move_events,
                        self.pointer_button_events,
                    ));
                    self.message = None;
                }
                Err(error) => self.message = Some(error.into()),
            }
        }
    }
    fn finish(&mut self, scale: f32, reason: &str) {
        let Some(report) = self.capture.stop() else {
            return;
        };
        let Some((began, start_scale, start_moves, start_buttons)) = self.started.take() else {
            self.fail("Capture session start was not recorded".into());
            return;
        };
        let record = serde_json::json!({
            "record_type":"session", "schema_version":1, "backend":self.backend,
            "finish_reason":reason, "elapsed_gui_ms":began.elapsed().as_millis(),
            "pixels_per_point_start":start_scale, "pixels_per_point_end":scale,
            "egui_pointer_move_events":self.pointer_move_events.saturating_sub(start_moves),
            "egui_pointer_button_events":self.pointer_button_events.saturating_sub(start_buttons),
            "report":report,
        });
        if let Err(error) = session::write_record(&mut self.output, &record) {
            self.fail(format!(
                "Capture output failed: {error}. The log is incomplete; choose a new file."
            ));
        }
    }
    fn fail(&mut self, message: String) {
        self.output_failed.store(true, Ordering::Release);
        self.message = Some(message);
    }
}
impl eframe::App for CaptureWindow {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let scale = ctx.pixels_per_point();
        let mut closing = false;
        ctx.input(|input| {
            self.pointer_position = input.pointer.hover_pos();
            // Preserve event order: a Space and a mouse event may share a frame.
            for event in &input.raw.events {
                match event {
                    egui::Event::PointerMoved { .. } => {
                        self.pointer_move_events = self.pointer_move_events.saturating_add(1);
                    }
                    egui::Event::PointerButton {
                        button, pressed, ..
                    } => {
                        self.pointer_button_events = self.pointer_button_events.saturating_add(1);
                        self.capture.push(*button, *pressed);
                    }
                    egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        ..
                    } => match key {
                        egui::Key::Space => self.toggle(scale),
                        egui::Key::Escape => {
                            self.finish(scale, "closed");
                            closing = true;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            closing |= input.viewport().close_requested();
        });
        if closing {
            self.finish(scale, "closed");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::Frame::new().fill(egui::Color32::from_rgb(22, 24, 29)).inner_margin(24.0).show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Input capture");
            ui.label(format!("Native backend: {} · scale {:.2}", self.backend, ui.ctx().pixels_per_point()));
            match self.pointer_position {
                Some(position) => ui.label(format!("Cursor in client: yes · ({:.1}, {:.1})", position.x, position.y)),
                None => ui.label("Cursor in client: no known position"),
            };
            ui.label(format!("egui mouse events: {} moves · {} button edges", self.pointer_move_events, self.pointer_button_events));
            ui.add_space(12.0);
            ui.colored_label(if self.capture.active() { egui::Color32::LIGHT_GREEN } else { egui::Color32::LIGHT_BLUE },
                if self.capture.active() { "Recording · Space ends this session" } else { "Ready · Space begins a new session" });
            ui.add_space(12.0);
            egui::Grid::new("capture-counts").striped(true).spacing([24.0, 12.0]).show(ui, |ui| {
                ui.strong("Button"); ui.strong("Down / up"); ui.strong("Pairs"); ui.strong("Held / anomalies"); ui.end_row();
                for (name, count) in session::BUTTONS.iter().zip(self.capture.counts()) {
                    ui.label(*name); ui.label(format!("{} / {}", count.presses, count.releases));
                    ui.label(count.pairs.to_string()); ui.label(format!("{} / {}", count.held, count.anomalies)); ui.end_row();
                }
            });
            ui.add_space(16.0);
            ui.label("Use Alt+Tab to change windows without adding a focus click.");
            ui.label("Arm with Space, start NanoClick using its global shortcut, then end with Space.");
            ui.label("Escape or closing saves an active session. A stopped session stays in the log.");
            ui.weak("Window events only. Counts do not establish native timing or global physical button state.");
            if let Some(issue) = self.capture.issue() { ui.colored_label(egui::Color32::LIGHT_RED, issue); }
            if let Some(message) = &self.message { ui.colored_label(egui::Color32::LIGHT_RED, message); }
            });
        });
    }
}
fn app_error(error: io::Error) -> eframe::Error {
    eframe::Error::AppCreation(Box::new(error))
}
fn main() -> eframe::Result {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or_else(|| app_error(io::Error::new(io::ErrorKind::InvalidInput, USAGE)))?;
    if args.next().is_some() {
        return Err(app_error(io::Error::new(
            io::ErrorKind::InvalidInput,
            USAGE,
        )));
    }
    if path == "--help" {
        println!("{USAGE}");
        return Ok(());
    }
    let output = BufWriter::new(session::create(Path::new(&path)).map_err(app_error)?);
    let output_failed = Arc::new(AtomicBool::new(false));
    let failure = output_failed.clone();
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([760.0, 480.0])
            .with_min_inner_size([640.0, 420.0]),
        ..Default::default()
    };
    options.renderer = renderer::selected();
    let renderer = renderer::name();
    eframe::run_native(
        "NanoClick input capture",
        options,
        Box::new(move |cc| {
            let backend = match cc.display_handle()?.as_raw() {
                RawDisplayHandle::Wayland(_) => "wayland",
                RawDisplayHandle::Xlib(_) | RawDisplayHandle::Xcb(_) => "x11",
                RawDisplayHandle::Windows(_) => "windows",
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "This capture tool supports Windows, X11 and Wayland only",
                    )
                    .into());
                }
            };
            cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
            cc.egui_ctx.all_styles_mut(|style| {
                style.visuals.override_text_color = Some(egui::Color32::from_rgb(224, 226, 230));
                style
                    .text_styles
                    .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
                style
                    .text_styles
                    .insert(egui::TextStyle::Heading, egui::FontId::proportional(26.0));
            });
            let mut output = output;
            session::write_record(
                &mut output,
                &serde_json::json!({
                    "record_type":"metadata", "schema_version":1, "app_version":env!("CARGO_PKG_VERSION"),
                    "backend":backend, "renderer":renderer, "button_order":session::BUTTONS,
                    "scope":"GUI mouse-button counts only; not native timing or global button-state proof",
                }),
            )?;
            Ok(Box::new(CaptureWindow {
                capture: session::Capture::default(),
                output,
                backend,
                started: None,
                message: None,
                output_failed: failure,
                pointer_position: None,
                pointer_move_events: 0,
                pointer_button_events: 0,
            }))
        }),
    )?;
    if output_failed.load(Ordering::Acquire) {
        return Err(app_error(io::Error::other(
            "Capture output failed; log is incomplete",
        )));
    }
    Ok(())
}
