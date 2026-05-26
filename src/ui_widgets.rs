//! Custom HUD widgets. For Task #01 only `card_frame` exists; later HUD
//! issues add chip strips, swatch rows, glass sliders, etc.

use crate::ui_tokens::{BORDER, CARD_BG, RADIUS_CARD};

/// Paints a dark-glass card background (rounded rect, `CARD_BG` fill, 1 px
/// `BORDER` stroke) and renders `contents` inside with 12 px horizontal /
/// 14 px vertical padding.
pub fn card_frame(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(CARD_BG)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(RADIUS_CARD)
        .inner_margin(egui::Margin::symmetric(12, 14))
        .show(ui, contents);
}
