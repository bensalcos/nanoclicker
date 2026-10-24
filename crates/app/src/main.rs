#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#![forbid(unsafe_code)]
mod controller;
mod hotkey;
mod theme;
mod ui;

use eframe::egui;
fn main() -> eframe::Result {
    let mut selected_theme = egui::ThemePreference::System;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--help" {
            println!("NanoClick\nUsage: nanoclick [--theme system|light|dark]");
            return Ok(());
        }
        let value = if argument == "--theme" {
            args.next()
        } else {
            None
        };
        selected_theme = match value.as_deref() {
            Some("system") => egui::ThemePreference::System,
            Some("light") => egui::ThemePreference::Light,
            Some("dark") => egui::ThemePreference::Dark,
            _ => {
                return Err(eframe::Error::AppCreation(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Usage: nanoclick [--theme system|light|dark]",
                ))));
            }
        };
    }
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 600.0])
            .with_min_inner_size([420.0, 520.0])
            .with_app_id("io.github.bensalcos.NanoClick"),
        ..Default::default()
    };
    #[cfg(feature = "glow")]
    {
        options.renderer = eframe::Renderer::Glow;
    }
    #[cfg(all(feature = "wgpu", not(feature = "glow")))]
    {
        options.renderer = eframe::Renderer::Wgpu;
    }
    eframe::run_native(
        "NanoClick",
        options,
        Box::new(|cc| Ok(Box::new(ui::NanoClick::new(cc, selected_theme)?))),
    )
}
