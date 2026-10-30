use super::{button_name, section};
use crate::controller::Draft;
use eframe::egui;
use nanoclick_core::MouseButton;

pub(super) fn plan(ui: &mut egui::Ui, draft: &mut Draft) {
    section(ui, "01", "Timing", |ui| {
        ui.horizontal(|ui| {
            let label = ui.label("Interval");
            ui.add(
                egui::TextEdit::singleline(&mut draft.interval)
                    .desired_width(104.0)
                    .char_limit(6),
            )
            .labelled_by(label.id);
            ui.label("ms");
            if let Ok(config) = draft.validate() {
                ui.weak(format!(
                    "≈ {:.1} clicks/s",
                    1.0 / config.interval().as_secs_f64()
                ));
            }
        });
        ui.horizontal(|ui| {
            let label = ui.label("Mouse button");
            let selector = egui::ComboBox::from_id_salt("mouse-button")
                .selected_text(button_name(draft.button))
                .show_ui(ui, |ui| {
                    for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
                        ui.selectable_value(&mut draft.button, button, button_name(button));
                    }
                });
            let response = selector.response.labelled_by(label.id);
            restore_focus_on_escape(&response, selector.inner.is_some());
            describe_selected_text(&response, button_name(draft.button));
        });
    });
    section(ui, "02", "Repeat", |ui| {
        ui.radio_value(&mut draft.finite, false, "Until stopped");
        ui.horizontal(|ui| {
            let label = ui.radio_value(&mut draft.finite, true, "Count");
            ui.add_enabled(
                draft.finite,
                egui::TextEdit::singleline(&mut draft.count)
                    .desired_width(104.0)
                    .hint_text("Clicks")
                    .char_limit(6),
            )
            .labelled_by(label.id);
            ui.weak("clicks");
        });
    });
}

pub(super) fn shortcut(ui: &mut egui::Ui, requested: &mut String) -> bool {
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(requested)
                .desired_width(160.0)
                .hint_text("Ctrl+Shift+F6"),
        )
        .accessible_name("Global shortcut");
        ui.button("Apply").clicked()
    })
    .inner
}

pub(super) fn theme(ui: &mut egui::Ui, selected: &mut egui::ThemePreference) {
    let selector = egui::ComboBox::from_id_salt("theme")
        .selected_text(theme_name(*selected))
        .show_ui(ui, |ui| {
            for (value, label) in [
                (egui::ThemePreference::System, "System"),
                (egui::ThemePreference::Light, "Light"),
                (egui::ThemePreference::Dark, "Dark"),
            ] {
                if ui.selectable_value(selected, value, label).changed() {
                    ui.ctx().set_theme(*selected);
                }
            }
        });
    let response = selector.response.accessible_name("Theme");
    restore_focus_on_escape(&response, selector.inner.is_some());
    describe_selected_text(&response, theme_name(*selected));
}

fn restore_focus_on_escape(response: &egui::Response, popup_was_visible: bool) {
    // Escape dismisses the popup without choosing another control. Keep keyboard
    // traversal at its trigger; pointer dismissal must retain the clicked target.
    if popup_was_visible
        && response
            .ctx
            .input(|input| input.key_pressed(egui::Key::Escape))
    {
        response.request_focus();
    }
}

fn theme_name(selected: egui::ThemePreference) -> &'static str {
    match selected {
        egui::ThemePreference::Dark => "Dark",
        egui::ThemePreference::Light => "Light",
        egui::ThemePreference::System => "System",
    }
}

fn describe_selected_text(response: &egui::Response, selected: &str) {
    // AT-SPI exposes only numeric Value and these collapsed selectors have no
    // selected child. Preserve the name/value and supply the text independently.
    if cfg!(target_os = "linux") {
        response.ctx.accesskit_node_builder(response.id, |node| {
            node.set_description(selected);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn name(
        node: &egui::accesskit::Node,
        nodes: &[(egui::accesskit::NodeId, egui::accesskit::Node)],
    ) -> String {
        if let Some(label) = node.label().or_else(|| {
            matches!(node.role(), egui::Role::Label | egui::Role::TextRun)
                .then(|| node.value())
                .flatten()
        }) {
            return label.to_owned();
        }
        node.labelled_by()
            .iter()
            .filter_map(|id| nodes.iter().find(|(candidate, _)| candidate == id))
            .map(|(_, label)| name(label, nodes))
            .collect::<Vec<_>>()
            .join(" ")
    }
    #[test]
    fn controls_expose_semantic_names_and_values_without_a_desktop()
    -> Result<(), Box<dyn std::error::Error>> {
        let ctx = egui::Context::default();
        crate::theme::configure(&ctx);
        ctx.enable_accesskit();
        let mut draft = Draft {
            finite: true,
            ..Default::default()
        };
        let mut requested = "F6".to_owned();
        let mut selected = egui::ThemePreference::Dark;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            plan(ui, &mut draft);
            shortcut(ui, &mut requested);
            theme(ui, &mut selected);
        });
        output.textures_delta.clear();
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or("AccessKit is enabled but produced no tree")?
            .nodes;
        for (role, label, value) in [
            (egui::Role::TextInput, "Interval", "100"),
            (egui::Role::TextInput, "Count", "10"),
            (egui::Role::TextInput, "Global shortcut", "F6"),
            (egui::Role::ComboBox, "Mouse button", "Left"),
            (egui::Role::ComboBox, "Theme", "Dark"),
        ] {
            let node = nodes
                .iter()
                .map(|(_, node)| node)
                .find(|node| node.role() == role && name(node, nodes) == label)
                .unwrap_or_else(|| panic!("Missing named {role:?}: {label}"));
            assert_eq!(
                node.value(),
                Some(value),
                "{label} must expose its current value"
            );
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn selectors_expose_current_text_to_native_atspi() -> Result<(), Box<dyn std::error::Error>> {
        for (button, selected, button_text, theme_text) in [
            (
                MouseButton::Left,
                egui::ThemePreference::System,
                "Left",
                "System",
            ),
            (
                MouseButton::Right,
                egui::ThemePreference::Light,
                "Right",
                "Light",
            ),
            (
                MouseButton::Middle,
                egui::ThemePreference::Dark,
                "Middle",
                "Dark",
            ),
        ] {
            let ctx = egui::Context::default();
            crate::theme::configure(&ctx);
            ctx.enable_accesskit();
            let mut draft = Draft {
                button,
                ..Default::default()
            };
            let mut selected = selected;
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                plan(ui, &mut draft);
                theme(ui, &mut selected);
            });
            output.textures_delta.clear();
            let nodes = &output
                .platform_output
                .accesskit_update
                .as_ref()
                .ok_or("Missing accessibility tree")?
                .nodes;
            for (label, value) in [("Mouse button", button_text), ("Theme", theme_text)] {
                let node = nodes
                    .iter()
                    .map(|(_, node)| node)
                    .find(|node| node.role() == egui::Role::ComboBox && name(node, nodes) == label)
                    .ok_or("Selector is missing its stable name")?;
                assert_eq!(node.value(), Some(value));
                assert_eq!(
                    node.description(),
                    Some(value),
                    "AT-SPI selectors need independently readable selected text"
                );
            }
        }
        Ok(())
    }
}
