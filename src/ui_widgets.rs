//! Custom HUD widgets. For Task #01 only `card_frame` exists; later HUD
//! issues add chip strips, swatch rows, glass sliders, etc.

use crate::ui_tokens::{
    ACCENT, ACCENT_DIM, BODY_SIZE, BORDER, CARD_BG, RADIUS_CARD, RADIUS_CHIP, SURFACE_MUTE,
    TEXT_PRIMARY,
};

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

/// Lighten a color's alpha by `delta` (clamped to [0, 255]). Used for chip hover.
fn lighten_alpha(c: egui::Color32, delta: f32) -> egui::Color32 {
    let [r, g, b, a] = c.to_array();
    let new_a = (a as f32 + delta * 255.0).clamp(0.0, 255.0) as u8;
    egui::Color32::from_rgba_unmultiplied(r, g, b, new_a)
}

/// Multiply a color's alpha by `mul` (clamped to [0, 255]). Used for chip disabled state.
fn scale_alpha(c: egui::Color32, mul: f32) -> egui::Color32 {
    let [r, g, b, a] = c.to_array();
    let new_a = (a as f32 * mul).clamp(0.0, 255.0) as u8;
    egui::Color32::from_rgba_unmultiplied(r, g, b, new_a)
}

/// Renders a horizontal row of chips. `enabled[i]` controls whether chip `i`
/// is clickable (disabled chips render at 25 % alpha and ignore input).
/// Returns `Some(index)` if the user clicked an enabled chip this frame.
///
/// `id_salt` distinguishes multiple chip rows within the same `Ui`.
pub fn chip_strip(
    ui: &mut egui::Ui,
    labels: &[&str],
    selected: usize,
    enabled: &[bool],
    id_salt: &str,
) -> Option<usize> {
    debug_assert_eq!(labels.len(), enabled.len());
    let mut clicked: Option<usize> = None;
    let font_id = egui::FontId::proportional(BODY_SIZE);

    ui.push_id(id_salt, |ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        ui.horizontal(|ui| {
            for (i, label) in labels.iter().enumerate() {
                let is_enabled = enabled[i];
                let is_selected = i == selected;

                // Measure text to size the chip (3 px vertical, 7 px horizontal padding).
                let galley = ui.painter().layout_no_wrap(
                    (*label).to_string(),
                    font_id.clone(),
                    TEXT_PRIMARY,
                );
                let chip_size = egui::vec2(galley.size().x + 14.0, galley.size().y + 6.0);

                let sense = if is_enabled {
                    egui::Sense::click()
                } else {
                    egui::Sense::hover()
                };
                let (rect, response) = ui.allocate_exact_size(chip_size, sense);
                let response = response.on_hover_cursor(if is_enabled {
                    egui::CursorIcon::PointingHand
                } else {
                    egui::CursorIcon::Default
                });

                // Pick fill/stroke based on state.
                let (mut fill, stroke, mut text_color) = if is_selected {
                    (
                        ACCENT_DIM,
                        egui::Stroke::new(1.0, ACCENT),
                        TEXT_PRIMARY,
                    )
                } else {
                    (SURFACE_MUTE, egui::Stroke::NONE, TEXT_PRIMARY)
                };

                if is_enabled && response.hovered() {
                    fill = lighten_alpha(fill, 0.04);
                }

                if !is_enabled {
                    // Disabled chips render at 25 % alpha across fill, stroke, and text.
                    fill = scale_alpha(fill, 0.25);
                    text_color = scale_alpha(text_color, 0.25);
                }

                let stroke = if !is_enabled {
                    egui::Stroke::new(stroke.width, scale_alpha(stroke.color, 0.25))
                } else {
                    stroke
                };

                let painter = ui.painter();
                painter.rect(
                    rect,
                    RADIUS_CHIP,
                    fill,
                    stroke,
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    *label,
                    font_id.clone(),
                    text_color,
                );

                if is_enabled && response.clicked() {
                    clicked = Some(i);
                }
            }
        });
    });

    clicked
}
