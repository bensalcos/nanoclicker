use eframe::egui::{self, Color32, FontId, TextStyle};
pub const BLUE: Color32 = Color32::from_rgb(68, 139, 245);
pub const RED: Color32 = Color32::from_rgb(200, 52, 65);
pub fn configure(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::System);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 6.0);
        style.spacing.button_padding = egui::vec2(14.0, 6.0);
        style.spacing.interact_size.y = 30.0;
        style.spacing.scroll.floating = false;
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(15.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(15.0));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(26.0));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.0));
        style.visuals.override_text_color = Some(if style.visuals.dark_mode {
            Color32::from_rgb(224, 226, 230)
        } else {
            Color32::from_rgb(32, 36, 42)
        });
        style.visuals.weak_text_color = Some(if style.visuals.dark_mode {
            Color32::from_rgb(174, 180, 190)
        } else {
            Color32::from_rgb(85, 92, 104)
        });
        let boundary = if style.visuals.dark_mode {
            Color32::from_rgb(133, 142, 156)
        } else {
            Color32::from_rgb(96, 106, 120)
        };
        let focus = if style.visuals.dark_mode {
            Color32::WHITE
        } else {
            Color32::from_rgb(32, 36, 42)
        };
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, boundary);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, boundary);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, focus);
        style.visuals.widgets.open.bg_stroke = egui::Stroke::new(2.0, focus);
        style.visuals.selection.stroke = egui::Stroke::new(2.0, focus);
        style.visuals.selection.bg_fill = if style.visuals.dark_mode {
            Color32::from_rgb(30, 100, 200)
        } else {
            BLUE
        };
        style.animation_time = 0.0;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn luminance(color: Color32) -> f64 {
        let linear = |value: u8| {
            let value = f64::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
    }
    fn contrast(a: Color32, b: Color32) -> f64 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
    #[test]
    fn themes_keep_text_control_boundaries_and_focus_identifiable() {
        let ctx = egui::Context::default();
        configure(&ctx);
        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            let style = ctx.style_of(theme);
            let visuals = &style.visuals;
            let panel = visuals.panel_fill;
            assert!(
                contrast(visuals.text_color(), panel) >= 4.5,
                "{theme:?}: body text"
            );
            assert!(
                contrast(visuals.weak_text_color(), panel) >= 4.5,
                "{theme:?}: secondary text"
            );
            for widget in [
                &visuals.widgets.inactive,
                &visuals.widgets.hovered,
                &visuals.widgets.active,
            ] {
                assert!(
                    widget.bg_stroke.width >= 1.0 && widget.bg_stroke.color.a() == 255,
                    "{theme:?}: visible control boundary"
                );
                assert!(
                    contrast(widget.bg_stroke.color, panel) >= 3.0,
                    "{theme:?}: boundary against panel"
                );
                assert!(
                    contrast(widget.bg_stroke.color, widget.weak_bg_fill) >= 3.0,
                    "{theme:?}: boundary against control fill"
                );
            }
            assert!(
                contrast(visuals.selection.stroke.color, visuals.text_edit_bg_color()) >= 3.0,
                "{theme:?}: text input focus"
            );
            assert!(
                contrast(visuals.selection.stroke.color, visuals.selection.bg_fill) >= 4.5,
                "{theme:?}: selected text"
            );
        }
        assert!(contrast(Color32::WHITE, Color32::from_rgb(30, 100, 200)) >= 4.5);
        assert!(contrast(Color32::WHITE, RED) >= 4.5);
    }
}
