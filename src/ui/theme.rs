use eframe::egui::{self, Color32, RichText, Rounding, Stroke, Vec2};

pub const BG_BLACK: Color32 = Color32::BLACK;
pub const BG_DARK: Color32 = Color32::from_rgb(13, 13, 13);
pub const TEXT_WHITE: Color32 = Color32::WHITE;
pub const TEXT_MUTED: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 153);
pub const TEXT_DISABLED: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 76);
pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 25);
pub const BORDER_SECONDARY: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 102);
pub const STATUS_GREEN: Color32 = Color32::from_rgb(16, 185, 129);
pub const STATUS_RED: Color32 = Color32::from_rgb(239, 68, 68);
pub const HOVER_BG: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 13);
pub fn apply_dark_theme(ctx: &egui::Context) {

    ctx.tessellation_options_mut(|options| {
        options.feathering = true;
        options.feathering_size_in_pixels = 1.0;
    });
    let mut style = (*ctx.style()).clone();

    style.visuals.dark_mode = true;
    style.visuals.panel_fill = BG_BLACK;
    style.visuals.window_fill = BG_BLACK;
    style.visuals.extreme_bg_color = BG_DARK;

    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 45);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(45, 45, 45);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, BORDER_SECONDARY);
    style.visuals.widgets.inactive.rounding = Rounding::same(8.0);
    style.visuals.widgets.hovered.bg_fill = HOVER_BG;
    style.visuals.widgets.hovered.weak_bg_fill = HOVER_BG;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_WHITE);
    style.visuals.widgets.hovered.rounding = Rounding::same(8.0);
    style.visuals.widgets.active.bg_fill = STATUS_GREEN;
    style.visuals.widgets.active.weak_bg_fill = STATUS_GREEN;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT_WHITE);
    style.visuals.widgets.active.rounding = Rounding::same(8.0);
    style.visuals.widgets.noninteractive.bg_fill = BG_BLACK;
    style.visuals.widgets.noninteractive.weak_bg_fill = BG_BLACK;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_MUTED);
    style.visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

    style.visuals.selection.bg_fill = STATUS_GREEN;
    style.visuals.selection.stroke = Stroke::new(2.0, STATUS_GREEN);

    style.visuals.override_text_color = Some(TEXT_WHITE);

    style.visuals.window_rounding = Rounding::same(12.0);
    style.visuals.menu_rounding = Rounding::same(8.0);

    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    ctx.set_style(style);
}
pub fn styled_button(ui: &mut egui::Ui, text: &str, primary: bool) -> egui::Response {
    let button = egui::Button::new(RichText::new(text).size(14.0).color(TEXT_WHITE))
        .min_size(Vec2::new(100.0, 36.0))
        .rounding(Rounding::same(18.0));
    let button = if primary {
        button
            .fill(BG_BLACK)
            .stroke(Stroke::new(2.0, TEXT_WHITE))
    } else {
        button
            .fill(BG_BLACK)
            .stroke(Stroke::new(1.5, BORDER_SECONDARY))
    };
    ui.add(button)
}
pub fn styled_text_edit(ui: &mut egui::Ui, text: &mut String, hint: &str) -> egui::Response {
    let text_edit = egui::TextEdit::singleline(text)
        .desired_width(ui.available_width())
        .hint_text(RichText::new(hint).color(TEXT_DISABLED))
        .margin(Vec2::new(12.0, 8.0))
        .text_color(TEXT_WHITE);
    egui::Frame::none()
        .fill(BG_BLACK)
        .stroke(Stroke::new(1.0, BORDER_SECONDARY))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.add(text_edit)
        })
        .inner
}
pub fn card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(BG_DARK)
        .rounding(Rounding::same(16.0))
        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
        .inner_margin(24.0)
}
pub fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(18.0).color(TEXT_WHITE).strong());
    ui.add_space(8.0);
}
pub fn status_dot(ui: &mut egui::Ui, running: bool) {
    let color = if running { STATUS_GREEN } else { STATUS_RED };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 6.0, color);
}
pub fn styled_checkbox(ui: &mut egui::Ui, checked: &mut bool, text: &str) -> egui::Response {
    ui.horizontal(|ui| {
        let response = ui.checkbox(checked, "");
        ui.label(RichText::new(text).color(TEXT_WHITE));
        response
    }).inner
}