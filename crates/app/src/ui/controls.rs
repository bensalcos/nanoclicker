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
            egui::ComboBox::from_id_salt("mouse-button")
                .selected_text(button_name(draft.button))
                .show_ui(ui, |ui| {
                    for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
                        ui.selectable_value(&mut draft.button, button, button_name(button));
                    }
                })
                .response
                .labelled_by(label.id);
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
    egui::ComboBox::from_id_salt("theme")
        .selected_text(match selected {
            egui::ThemePreference::Dark => "Dark",
            egui::ThemePreference::Light => "Light",
            _ => "System",
        })
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
        })
        .response
        .accessible_name("Theme");
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
}
