#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#![forbid(unsafe_code)]
mod cli;
mod controller;
mod diagnostics;
mod hotkey;
mod renderer;
mod settings;
mod theme;
mod ui;

use eframe::egui;
fn main() -> eframe::Result {
    let selected = cli::parse(std::env::args_os().skip(1)).map_err(|message| {
        eframe::Error::AppCreation(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            message,
        )))
    })?;
    if selected.help {
        println!("NanoClick\n{}", cli::USAGE);
        return Ok(());
    }
    let diagnostics = selected
        .diagnostics
        .as_deref()
        .map(diagnostics::Diagnostics::create)
        .transpose()
        .map_err(|error| eframe::Error::AppCreation(Box::new(error)))?;
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 600.0])
            .with_min_inner_size([420.0, 520.0])
            .with_app_id("io.github.bensalcos.NanoClick"),
        ..Default::default()
    };
    options.renderer = renderer::selected();
    eframe::run_native(
        "NanoClick",
        options,
        Box::new(move |cc| {
            Ok(Box::new(ui::NanoClick::new(
                cc,
                selected.theme,
                diagnostics,
            )?))
        }),
    )
}
