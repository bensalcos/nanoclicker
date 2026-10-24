use eframe::egui::{self, Color32, FontId, TextStyle};
pub const BLUE: Color32 = Color32::from_rgb(68, 139, 245);
pub const RED: Color32 = Color32::from_rgb(200, 52, 65);
pub fn configure(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::System);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 6.0);
        style.spacing.button_padding = egui::vec2(14.0, 9.0);
        style.spacing.interact_size.y = 30.0;
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
        style.visuals.selection.bg_fill = BLUE;
        style.animation_time = 0.0;
    });
}
