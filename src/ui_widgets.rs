//! Custom HUD widgets. For Task #01 only `card_frame` exists; later HUD
//! issues add chip strips, swatch rows, glass sliders, etc.

use crate::ui_tokens::{
    ACCENT, ACCENT_DIM, BODY_SIZE, BORDER, CARD_BG, LABEL_SIZE, RADIUS_CARD, RADIUS_CHIP,
    RADIUS_PILL, SURFACE_MUTE, TEXT_PRIMARY, TEXT_TERTIARY,
};

/// Swatch height in pixels (matches v4/v5 mockups).
const SWATCH_H: f32 = 12.0;

/// Number of horizontal sub-rects per swatch when painting the gradient.
const SWATCH_STOPS: usize = 16;

/// Corner radius for swatches — slightly tighter than chips so they read as
/// a different control class.
const SWATCH_RADIUS: f32 = 2.0;

/// Horizontal gap between adjacent swatches.
const SWATCH_GAP: f32 = 4.0;

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

/// Linearly interpolate two `[u8; 3]` stops at fraction `t` in [0, 1].
fn lerp_stop(a: [u8; 3], b: [u8; 3], t: f32) -> egui::Color32 {
    let lerp = |x: u8, y: u8| -> u8 {
        (x as f32 + (y as f32 - x as f32) * t).round().clamp(0.0, 255.0) as u8
    };
    egui::Color32::from_rgb(lerp(a[0], b[0]), lerp(a[1], b[1]), lerp(a[2], b[2]))
}

/// Sample a colormap LUT (an array of `[u8; 3]` stops) at fraction `t` in [0, 1].
fn sample_lut(stops: &[[u8; 3]], t: f32) -> egui::Color32 {
    if stops.is_empty() {
        return egui::Color32::BLACK;
    }
    if stops.len() == 1 {
        let s = stops[0];
        return egui::Color32::from_rgb(s[0], s[1], s[2]);
    }
    let t = t.clamp(0.0, 1.0);
    let scaled = t * (stops.len() - 1) as f32;
    let i = scaled.floor() as usize;
    let i = i.min(stops.len() - 2);
    let frac = scaled - i as f32;
    lerp_stop(stops[i], stops[i + 1], frac)
}

/// Renders a horizontal row of colormap gradient swatches. Each swatch shows
/// its colormap LUT as a left-to-right gradient. Returns `Some(index)` if the
/// user clicked a swatch this frame.
///
/// `luts` is the slice from `colormaps::ALL` (or any `&[(&str, &[[u8; 3]])]`).
/// The widget iterates the slice, so adding a colormap to `colormaps.rs` later
/// automatically extends the row.
pub fn swatch_row(
    ui: &mut egui::Ui,
    luts: &[(&str, &[[u8; 3]])],
    selected: usize,
) -> Option<usize> {
    let mut clicked: Option<usize> = None;
    if luts.is_empty() {
        return clicked;
    }

    let n = luts.len() as f32;
    let avail_w = ui.available_width();
    let total_gap = SWATCH_GAP * (n - 1.0).max(0.0);
    // Leave 1 px of slack so the ACCENT outer ring on the rightmost swatch
    // never gets clipped by the available rect.
    let swatch_w = ((avail_w - total_gap - 1.0).max(1.0)) / n;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SWATCH_GAP;
        for (i, (_name, stops)) in luts.iter().enumerate() {
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(swatch_w, SWATCH_H),
                egui::Sense::click(),
            );
            let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

            // Paint the gradient as SWATCH_STOPS sub-rects so we don't depend
            // on `epaint::Mesh`. 16 stops across ~50 px is visually smooth.
            let painter = ui.painter();
            let w = rect.width() / SWATCH_STOPS as f32;
            for k in 0..SWATCH_STOPS {
                let t0 = k as f32 / SWATCH_STOPS as f32;
                let t1 = (k + 1) as f32 / SWATCH_STOPS as f32;
                let t_mid = 0.5 * (t0 + t1);
                let color = sample_lut(stops, t_mid);
                let sub = egui::Rect::from_min_max(
                    egui::pos2(rect.left() + k as f32 * w, rect.top()),
                    egui::pos2(rect.left() + (k + 1) as f32 * w, rect.bottom()),
                );
                painter.rect_filled(sub, 0.0, color);
            }

            // Note: sub-rects above are square and ignore SWATCH_RADIUS. At
            // 2 px radius on a 12 px-tall strip the visual difference is
            // imperceptible against the card background; the selection
            // strokes below DO honor the radius and visually define the
            // swatch's rounded corners.

            if i == selected {
                // 1 px white inset stroke (sits INSIDE the gradient).
                painter.rect_stroke(
                    rect,
                    SWATCH_RADIUS,
                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                    egui::StrokeKind::Inside,
                );
                // 1 px ACCENT outer ring (sits OUTSIDE the gradient).
                painter.rect_stroke(
                    rect,
                    SWATCH_RADIUS,
                    egui::Stroke::new(1.0, ACCENT),
                    egui::StrokeKind::Outside,
                );
            }

            if response.clicked() {
                clicked = Some(i);
            }
        }
    });

    clicked
}

/// Outer pill size for the toggle switch.
const TOGGLE_W: f32 = 24.0;
const TOGGLE_H: f32 = 14.0;
/// Inner circle diameter and inset from the pill edge.
const TOGGLE_CIRCLE_D: f32 = 10.0;
const TOGGLE_INSET: f32 = 2.0;

/// Renders a 24x14 px pill toggle switch. Mutates `on` in place and returns the
/// `Response` so the caller can react to `.clicked()` / `.changed()`.
///
/// Off: `SURFACE_MUTE` fill, white circle inset from the left.
/// On:  `ACCENT_DIM` fill, `ACCENT` circle inset from the right.
pub fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let (rect, mut response) =
        ui.allocate_exact_size(egui::vec2(TOGGLE_W, TOGGLE_H), egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    let painter = ui.painter();
    let fill = if *on { ACCENT_DIM } else { SURFACE_MUTE };
    painter.rect(
        rect,
        RADIUS_PILL,
        fill,
        egui::Stroke::NONE,
        egui::StrokeKind::Inside,
    );

    let circle_color = if *on { ACCENT } else { egui::Color32::WHITE };
    let r = TOGGLE_CIRCLE_D * 0.5;
    let cx = if *on {
        rect.right() - TOGGLE_INSET - r
    } else {
        rect.left() + TOGGLE_INSET + r
    };
    let cy = rect.center().y;
    painter.circle_filled(egui::pos2(cx, cy), r, circle_color);

    response
}

/// Padding for action buttons.
const ACTION_PAD_X: f32 = 8.0;
const ACTION_PAD_Y: f32 = 6.0;
/// Gap between the main label and the optional shortcut hint.
const ACTION_HINT_GAP: f32 = 6.0;

/// Renders an action button with `label` and an optional inline keyboard
/// `shortcut` hint (painted in `TEXT_TERTIARY` at `LABEL_SIZE`). Returns the
/// `Response` so the caller can check `.clicked()`.
pub fn action_button(
    ui: &mut egui::Ui,
    label: &str,
    shortcut: Option<&str>,
) -> egui::Response {
    let label_font = egui::FontId::proportional(BODY_SIZE);
    let hint_font = egui::FontId::proportional(LABEL_SIZE);

    // Lay out the two text segments up-front so we can size the button and
    // paint them at the right positions afterwards.
    let label_galley = ui.painter().layout_no_wrap(
        label.to_string(),
        label_font.clone(),
        TEXT_PRIMARY,
    );
    let hint_galley = shortcut.map(|s| {
        ui.painter().layout_no_wrap(
            s.to_string(),
            hint_font.clone(),
            TEXT_TERTIARY,
        )
    });

    let text_w = label_galley.size().x
        + hint_galley
            .as_ref()
            .map(|g| ACTION_HINT_GAP + g.size().x)
            .unwrap_or(0.0);
    let text_h = label_galley
        .size()
        .y
        .max(hint_galley.as_ref().map(|g| g.size().y).unwrap_or(0.0));

    // Grow the button to fill the available horizontal width so a row of
    // buttons inside `ui.horizontal` splits the line evenly.
    let desired_w = (text_w + 2.0 * ACTION_PAD_X).max(ui.available_width());
    let desired_h = text_h + 2.0 * ACTION_PAD_Y;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(desired_w, desired_h), egui::Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    let pressed = response.is_pointer_button_down_on();
    let hovered = response.hovered() && !pressed;

    let (fill, stroke) = if pressed {
        (ACCENT_DIM, egui::Stroke::new(1.0, ACCENT))
    } else if hovered {
        (lighten_alpha(SURFACE_MUTE, 0.04), egui::Stroke::NONE)
    } else {
        (SURFACE_MUTE, egui::Stroke::NONE)
    };

    let painter = ui.painter();
    painter.rect(rect, RADIUS_CHIP, fill, stroke, egui::StrokeKind::Inside);

    // Center the label+hint composite horizontally inside the button.
    let start_x = rect.center().x - text_w * 0.5;
    let label_pos = egui::pos2(start_x, rect.center().y - label_galley.size().y * 0.5);
    painter.galley(label_pos, label_galley.clone(), TEXT_PRIMARY);

    if let Some(g) = hint_galley {
        let hint_x = start_x + label_galley.size().x + ACTION_HINT_GAP;
        let hint_pos = egui::pos2(hint_x, rect.center().y - g.size().y * 0.5);
        painter.galley(hint_pos, g, TEXT_TERTIARY);
    }

    response
}
