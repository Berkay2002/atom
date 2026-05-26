//! Custom HUD widgets. For Task #01 only `card_frame` exists; later HUD
//! issues add chip strips, swatch rows, glass sliders, etc.

use crate::ui_tokens::{
    ACCENT, ACCENT_DIM, BODY_SIZE, BORDER, CARD_BG, CARD_BG_LIGHT, EDGE_INSET, LABEL_SIZE,
    RADIUS_CARD, RADIUS_CHIP, RADIUS_PILL, SURFACE_MUTE, TEXT_PRIMARY, TEXT_SECONDARY,
    TEXT_TERTIARY,
};

/// Width/height of the eye toggle square, in px.
const EYE_W: f32 = 28.0;

/// Gap between the eye toggle and the hud pill, in px.
const PILL_EYE_GAP: f32 = 12.0;

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

/// Horizontal gap between adjacent chips.
const CHIP_GAP: f32 = 3.0;

/// Renders a horizontal row of chips. `enabled[i]` controls whether chip `i`
/// is clickable (disabled chips render at 25 % alpha and ignore input).
/// Returns `Some(index)` if the user clicked an enabled chip this frame.
///
/// Chips are equal-width and the row fills `ui.available_width()`, so every
/// chip strip in the card aligns to the same right edge as the colormap row
/// and the sliders.
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
    let n = labels.len();
    if n == 0 {
        return None;
    }
    let mut clicked: Option<usize> = None;
    let font_id = egui::FontId::proportional(BODY_SIZE);

    let avail_w = ui.available_width();
    let total_gap = CHIP_GAP * (n.saturating_sub(1)) as f32;
    let chip_w = ((avail_w - total_gap) / n as f32).max(1.0);

    ui.push_id(id_salt, |ui| {
        ui.spacing_mut().item_spacing.x = CHIP_GAP;
        ui.horizontal(|ui| {
            for (i, label) in labels.iter().enumerate() {
                let is_enabled = enabled[i];
                let is_selected = i == selected;

                let galley = ui.painter().layout_no_wrap(
                    (*label).to_string(),
                    font_id.clone(),
                    TEXT_PRIMARY,
                );
                let chip_size = egui::vec2(chip_w, galley.size().y + 6.0);

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

/// Chevron button size (square).
const CHEVRON_SIZE: f32 = 18.0;

/// Renders a small 18x18 chevron button used to collapse/expand the HUD card.
/// Shows `−` (U+2212) when `*expanded == true` and `+` when collapsed. Clicking
/// the button toggles `*expanded`. Returns the `Response` so callers can react
/// to `.clicked()` / `.changed()`.
pub fn chevron_button(ui: &mut egui::Ui, expanded: &mut bool) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(
        egui::vec2(CHEVRON_SIZE, CHEVRON_SIZE),
        egui::Sense::click(),
    );
    if response.clicked() {
        *expanded = !*expanded;
        response.mark_changed();
    }
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    let hovered = response.hovered();
    let fill = if hovered {
        lighten_alpha(SURFACE_MUTE, 0.04)
    } else {
        SURFACE_MUTE
    };
    let glyph_color = if hovered { TEXT_PRIMARY } else { TEXT_SECONDARY };

    let painter = ui.painter();
    painter.rect(
        rect,
        RADIUS_CHIP,
        fill,
        egui::Stroke::NONE,
        egui::StrokeKind::Inside,
    );
    let glyph = if *expanded { "\u{2212}" } else { "+" };
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(BODY_SIZE),
        glyph_color,
    );

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

/// Width of the left label column in a `glass_slider` row, in px. Sized so the
/// widest label (`saturation`) at `BODY_SIZE` leaves a comfortable gap to the
/// track, and so the gap matches the narrower `exposure` row exactly.
const SLIDER_LABEL_W: f32 = 84.0;

/// Width of the right numeric-value column in a `glass_slider` row, in px.
const SLIDER_VALUE_W: f32 = 36.0;

/// Total row height for the slider track region. The track itself is 3 px
/// tall but the knob (radius 4) plus glow (radius 7) need vertical headroom.
const SLIDER_ROW_H: f32 = 12.0;

/// Track thickness in px.
const SLIDER_TRACK_H: f32 = 3.0;

/// Knob radius in px.
const SLIDER_KNOB_R: f32 = 4.0;

/// Outer glow radius in px (paints `ACCENT_DIM` behind the solid knob).
const SLIDER_GLOW_R: f32 = 7.0;

/// Renders a labeled glass slider. Mutates `value` in [min, max]. Returns
/// the `Response` for the interactive track region so the caller can check
/// `.changed()`.
///
/// Layout (`ui.horizontal`):
///   [60 px label][stretch track + knob][36 px right-aligned value]
///
/// Interaction: drag or click anywhere on the track sub-region to scrub.
pub fn glass_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    display_decimals: usize,
) -> egui::Response {
    let min = *range.start();
    let max = *range.end();
    let value_font = egui::FontId::proportional(BODY_SIZE);
    // Slider row labels are per-control names, not section headers — render
    // them at body size so they match the right-side numeric readout.
    let label_font = egui::FontId::proportional(BODY_SIZE);

    // Compute the track width deterministically from the OUTER row width up
    // front, before any allocations happen inside the horizontal. Reading
    // `ui.available_width()` after intermediate allocations leaks egui's
    // subpixel rounding between rows, which made `exposure` and `saturation`
    // tracks differ by 1–2 px. Pre-computing fixes that.
    let row_w = ui.available_width();
    let item_spacing = ui.spacing().item_spacing.x;
    let track_w =
        (row_w - SLIDER_LABEL_W - SLIDER_VALUE_W - 2.0 * item_spacing).max(1.0);

    // Capture the response from the inner-most `horizontal` so we can return
    // it. We hold it in an `Option` populated by the closure.
    let mut out_response: Option<egui::Response> = None;

    ui.horizontal(|ui| {
        // ---- Left: label ----------------------------------------------------
        let (label_rect, _) =
            ui.allocate_exact_size(egui::vec2(SLIDER_LABEL_W, SLIDER_ROW_H), egui::Sense::hover());
        ui.painter().text(
            egui::pos2(label_rect.left(), label_rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            label_font.clone(),
            TEXT_TERTIARY,
        );

        // ---- Middle: track + knob ------------------------------------------
        let (track_rect, response) = ui.allocate_exact_size(
            egui::vec2(track_w, SLIDER_ROW_H),
            egui::Sense::click_and_drag(),
        );
        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        // Handle interaction: clicking or dragging anywhere in the track
        // region snaps/scrubs the value. We use the latest interact pointer
        // position from the response so click and drag share one code path.
        let denom = (max - min).max(f32::EPSILON);
        if response.clicked() || response.dragged() {
            if let Some(p) = response.interact_pointer_pos() {
                let span = (track_rect.right() - track_rect.left()).max(1.0);
                let t = ((p.x - track_rect.left()) / span).clamp(0.0, 1.0);
                let new_v = min + t * denom;
                if (new_v - *value).abs() > f32::EPSILON {
                    *value = new_v;
                }
            }
        }

        // Clamp once after possibly being mutated externally (or by us).
        let clamped = value.clamp(min, max);
        if (clamped - *value).abs() > f32::EPSILON {
            *value = clamped;
        }
        let t = ((*value - min) / denom).clamp(0.0, 1.0);

        // Track sub-rect: 3 px tall, centered vertically.
        let mid_y = track_rect.center().y;
        let bar_top = mid_y - SLIDER_TRACK_H * 0.5;
        let bar_bot = mid_y + SLIDER_TRACK_H * 0.5;
        let bar_rect = egui::Rect::from_min_max(
            egui::pos2(track_rect.left(), bar_top),
            egui::pos2(track_rect.right(), bar_bot),
        );
        let painter = ui.painter();
        painter.rect_filled(bar_rect, 1.0, SURFACE_MUTE);

        // Fill: left edge → knob, ACCENT_DIM.
        let knob_x = track_rect.left() + t * (track_rect.right() - track_rect.left());
        let fill_rect = egui::Rect::from_min_max(
            egui::pos2(track_rect.left(), bar_top),
            egui::pos2(knob_x, bar_bot),
        );
        painter.rect_filled(fill_rect, 1.0, ACCENT_DIM);

        // Glow first, then solid knob on top.
        let knob_pos = egui::pos2(knob_x, mid_y);
        painter.circle_filled(knob_pos, SLIDER_GLOW_R, ACCENT_DIM);
        painter.circle_filled(knob_pos, SLIDER_KNOB_R, ACCENT);

        // ---- Right: numeric value, right-aligned ----------------------------
        let (value_rect, _) = ui.allocate_exact_size(
            egui::vec2(SLIDER_VALUE_W, SLIDER_ROW_H),
            egui::Sense::hover(),
        );
        let text = format!("{:.*}", display_decimals, *value);
        ui.painter().text(
            egui::pos2(value_rect.right(), value_rect.center().y),
            egui::Align2::RIGHT_CENTER,
            text,
            value_font,
            TEXT_PRIMARY,
        );

        out_response = Some(response);
    });

    out_response.expect("horizontal closure always populates response")
}

/// Format `peak |ψ|²` for the hud pill. Uses scientific notation with one
/// fractional digit and a Unicode minus sign for the exponent when negative.
fn format_peak(value: f64) -> String {
    let raw = format!("{:.1e}", value);
    raw.replace('-', "\u{2212}")
}

/// Renders the top-right HUD pill combining the live FPS readout and the
/// current peak |ψ|² value. Self-contained `Area`; anchored to the top-right
/// of the viewport with `EDGE_INSET + EYE_W + PILL_EYE_GAP` of right inset so
/// it sits to the left of the eye toggle.
pub fn hud_pill(ctx: &egui::Context, fps: f32, peak: f64) {
    let right_inset = EDGE_INSET + EYE_W + PILL_EYE_GAP;
    egui::Area::new(egui::Id::new("hud-pill"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-right_inset, EDGE_INSET))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(CARD_BG_LIGHT)
                .stroke(egui::Stroke::new(1.0, BORDER))
                .corner_radius(RADIUS_PILL)
                .inner_margin(egui::Margin::symmetric(12, 5))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.horizontal(|ui| {
                        // Accent dot with subtle outer glow: paint a larger
                        // ACCENT_DIM circle behind the 6 px ACCENT dot.
                        let dot_d = 12.0_f32;
                        let (dot_rect, _) = ui.allocate_exact_size(
                            egui::vec2(dot_d, dot_d),
                            egui::Sense::hover(),
                        );
                        let center = dot_rect.center();
                        let painter = ui.painter();
                        painter.circle_filled(center, 5.0, ACCENT_DIM);
                        painter.circle_filled(center, 3.0, ACCENT);

                        // FPS value
                        ui.label(
                            egui::RichText::new(format!("{:.1}", fps))
                                .size(BODY_SIZE)
                                .color(TEXT_PRIMARY),
                        );
                        // " FPS" label
                        ui.label(
                            egui::RichText::new("FPS")
                                .size(LABEL_SIZE)
                                .color(TEXT_TERTIARY),
                        );

                        // Faint vertical separator.
                        let sep_h = BODY_SIZE + 2.0;
                        let (sep_rect, _) = ui.allocate_exact_size(
                            egui::vec2(1.0, sep_h),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(sep_rect, 0.0, BORDER);

                        // "peak |ψ|²" label
                        ui.label(
                            egui::RichText::new("peak |\u{03c8}|\u{00b2}")
                                .size(LABEL_SIZE)
                                .color(TEXT_TERTIARY),
                        );
                        // peak value
                        ui.label(
                            egui::RichText::new(format_peak(peak))
                                .size(BODY_SIZE)
                                .color(TEXT_PRIMARY),
                        );
                    });
                });
        });
}

/// Vertical padding inside a preset chip, in px.
const PRESET_CHIP_PAD_Y: f32 = 4.0;

/// Horizontal padding inside a preset chip, in px.
const PRESET_CHIP_PAD_X: f32 = 9.0;

/// Horizontal gap between adjacent preset chips, in px.
const PRESET_CHIP_GAP: f32 = 6.0;

/// Approximate width to reserve for the scale readout on the bottom-left so the
/// preset strip on the bottom-right doesn't overlap it. The bar is 120 px, plus
/// inline text. ~260 px is a safe conservative estimate.
const SCALE_READOUT_RESERVE: f32 = 260.0;

/// Sub-helper: lay out a single preset chip's rect (allocate + sense), paint
/// background/text, and return whether it was clicked this frame. `selected`
/// drives the highlight (accent fill + stroke).
fn paint_preset_chip(
    ui: &mut egui::Ui,
    label: &str,
    selected: bool,
) -> bool {
    let font_id = egui::FontId::proportional(BODY_SIZE);
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        font_id.clone(),
        TEXT_PRIMARY,
    );
    let chip_size = egui::vec2(
        galley.size().x + 2.0 * PRESET_CHIP_PAD_X,
        galley.size().y + 2.0 * PRESET_CHIP_PAD_Y,
    );
    let (rect, response) = ui.allocate_exact_size(chip_size, egui::Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    let (mut fill, stroke) = if selected {
        (ACCENT_DIM, egui::Stroke::new(1.0, ACCENT))
    } else {
        (CARD_BG_LIGHT, egui::Stroke::new(1.0, BORDER))
    };
    if response.hovered() && !selected {
        fill = lighten_alpha(fill, 0.05);
    }

    let painter = ui.painter();
    painter.rect(rect, RADIUS_PILL, fill, stroke, egui::StrokeKind::Inside);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        font_id,
        TEXT_PRIMARY,
    );

    response.clicked()
}

/// Estimate the natural width (in px) the strip would need to render all preset
/// chips inline at the current font.
fn estimate_preset_strip_width(
    ctx: &egui::Context,
    presets: &[(&str, u32, u32, i32)],
) -> f32 {
    let font_id = egui::FontId::proportional(BODY_SIZE);
    let n = presets.len() as f32;
    let total_gap = PRESET_CHIP_GAP * (n - 1.0).max(0.0);
    let chip_text_w: f32 = presets
        .iter()
        .map(|p| {
            ctx.fonts_mut(|f| {
                f.layout_no_wrap(p.0.to_string(), font_id.clone(), TEXT_PRIMARY)
                    .size()
                    .x
            })
        })
        .sum();
    let chip_padding = 2.0 * PRESET_CHIP_PAD_X * n;
    chip_text_w + chip_padding + total_gap
}

/// Renders the bottom-right preset chip strip. Each chip corresponds to one
/// entry in `presets`; clicking applies that entry's `(n, l, m)` to the mutable
/// references. Returns `true` if a chip was clicked this frame (caller should
/// trigger a rebake).
///
/// Overflow strategy: if the natural strip width would exceed
/// `screen_w - 2*EDGE_INSET - SCALE_READOUT_RESERVE`, only the chips that fit
/// are rendered inline, followed by a `More…` chip that opens a popover (a
/// second `Area` painted just above the strip) containing the remaining chips.
pub fn preset_strip(
    ctx: &egui::Context,
    presets: &[(&str, u32, u32, i32)],
    n: &mut u32,
    l: &mut u32,
    m: &mut i32,
) -> bool {
    if presets.is_empty() {
        return false;
    }
    let mut changed = false;

    // Per-chip widths (matches paint_preset_chip's chip_size calc).
    let font_id = egui::FontId::proportional(BODY_SIZE);
    let chip_widths: Vec<f32> = presets
        .iter()
        .map(|p| {
            let text_w = ctx.fonts_mut(|f| {
                f.layout_no_wrap(p.0.to_string(), font_id.clone(), TEXT_PRIMARY)
                    .size()
                    .x
            });
            text_w + 2.0 * PRESET_CHIP_PAD_X
        })
        .collect();
    let more_w = ctx.fonts_mut(|f| {
        f.layout_no_wrap("More\u{2026}".to_string(), font_id.clone(), TEXT_PRIMARY)
            .size()
            .x
    }) + 2.0 * PRESET_CHIP_PAD_X;

    let screen_w = ctx.screen_rect().width();
    let natural_w = estimate_preset_strip_width(ctx, presets);
    let available_w = (screen_w - 2.0 * EDGE_INSET - SCALE_READOUT_RESERVE).max(0.0);

    // Decide split: how many leading chips render inline.
    let (visible_count, needs_more) = if natural_w <= available_w {
        (presets.len(), false)
    } else {
        // Greedily fit chips + reserve room for the More chip.
        let mut used = 0.0_f32;
        let mut count = 0usize;
        for (i, w) in chip_widths.iter().enumerate() {
            let gap = if i == 0 { 0.0 } else { PRESET_CHIP_GAP };
            let next = used + gap + w + PRESET_CHIP_GAP + more_w;
            if next > available_w {
                break;
            }
            used += gap + w;
            count += 1;
        }
        (count, true)
    };

    let popup_id = egui::Id::new("preset-strip-more-popup");
    let mut popup_open: bool =
        ctx.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));

    egui::Area::new(egui::Id::new("preset-strip"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-EDGE_INSET, -EDGE_INSET))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.x = PRESET_CHIP_GAP;
            ui.horizontal(|ui| {
                for (i, p) in presets.iter().take(visible_count).enumerate() {
                    let selected = p.1 == *n && p.2 == *l && p.3 == *m;
                    if paint_preset_chip(ui, p.0, selected) {
                        *n = p.1;
                        *l = p.2;
                        *m = p.3;
                        changed = true;
                        popup_open = false;
                        let _ = i;
                    }
                }
                if needs_more {
                    // The "More…" chip is highlighted if the current selection
                    // lives in the overflow tail (so users can still see where
                    // their orbital lives when scrolled out of view).
                    let tail_has_selection = presets[visible_count..]
                        .iter()
                        .any(|p| p.1 == *n && p.2 == *l && p.3 == *m);
                    if paint_preset_chip(ui, "More\u{2026}", tail_has_selection) {
                        popup_open = !popup_open;
                    }
                }
            });
        });

    if needs_more && popup_open {
        // The popover is its own `Area`, painted just above the strip and
        // right-aligned with it. The tail chips wrap inside the popup.
        egui::Area::new(egui::Id::new("preset-strip-popup"))
            .anchor(
                egui::Align2::RIGHT_BOTTOM,
                egui::vec2(-EDGE_INSET, -(EDGE_INSET + 36.0)),
            )
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let max_w = (screen_w - 2.0 * EDGE_INSET).min(360.0);
                ui.set_max_width(max_w);
                egui::Frame::new()
                    .fill(CARD_BG)
                    .stroke(egui::Stroke::new(1.0, BORDER))
                    .corner_radius(RADIUS_CARD)
                    .inner_margin(egui::Margin::symmetric(10, 10))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing =
                            egui::vec2(PRESET_CHIP_GAP, PRESET_CHIP_GAP);
                        ui.horizontal_wrapped(|ui| {
                            for p in presets.iter().skip(visible_count) {
                                let selected =
                                    p.1 == *n && p.2 == *l && p.3 == *m;
                                if paint_preset_chip(ui, p.0, selected) {
                                    *n = p.1;
                                    *l = p.2;
                                    *m = p.3;
                                    changed = true;
                                    popup_open = false;
                                }
                            }
                        });
                    });
            });
    }

    ctx.data_mut(|d| d.insert_temp(popup_id, popup_open));

    changed
}

/// Renders the bottom-left scale readout: a 120 px white bar with end caps,
/// the inline `{a₀} · {nm}` measurement, and a small-cap `BOX ±N a₀` label.
/// Both the bar and the text get a subtle dark drop-shadow for legibility over
/// bright orbital lobes (no card behind this widget).
///
/// Math: bar length in a₀ = bar_px · a₀-per-px, where a₀-per-px is derived from
/// the camera's 60° vertical FOV. nm = a₀ · 0.0529177. `box_half` is rendered
/// verbatim in the BOX label.
pub fn scale_readout(ctx: &egui::Context, camera_radius: f32, box_half: f64) {
    let bar_px = 120.0_f32;
    let viewport_h = ctx.screen_rect().height();
    let visible_world_h = 2.0 * camera_radius * (60.0_f32.to_radians() * 0.5).tan();
    let a0_per_px = visible_world_h / viewport_h;
    let bar_a0 = (bar_px * a0_per_px) as f64;
    let bar_nm = bar_a0 * 0.052_917_7;

    egui::Area::new(egui::Id::new("scale"))
        .anchor(
            egui::Align2::LEFT_BOTTOM,
            egui::vec2(EDGE_INSET, -EDGE_INSET),
        )
        .show(ctx, |ui| {
            // ---- Bar with end caps + shadow -------------------------------------
            let (response, painter) = ui.allocate_painter(
                egui::vec2(bar_px, 14.0),
                egui::Sense::hover(),
            );
            let rect = response.rect;
            let mid_y = rect.center().y;
            let cap_half = 4.0_f32;
            let shadow = egui::Color32::from_black_alpha(128);
            let white = egui::Color32::WHITE;
            let stroke_w = 1.5_f32;

            // Shadow pass: 1 px offset down/right, black @ 50% alpha.
            let off = egui::vec2(1.0, 1.0);
            painter.line_segment(
                [
                    egui::pos2(rect.left() + off.x, mid_y + off.y),
                    egui::pos2(rect.right() + off.x, mid_y + off.y),
                ],
                egui::Stroke { width: stroke_w, color: shadow },
            );
            painter.line_segment(
                [
                    egui::pos2(rect.left() + off.x, mid_y - cap_half + off.y),
                    egui::pos2(rect.left() + off.x, mid_y + cap_half + off.y),
                ],
                egui::Stroke { width: stroke_w, color: shadow },
            );
            painter.line_segment(
                [
                    egui::pos2(rect.right() + off.x, mid_y - cap_half + off.y),
                    egui::pos2(rect.right() + off.x, mid_y + cap_half + off.y),
                ],
                egui::Stroke { width: stroke_w, color: shadow },
            );

            // Main pass: white bar + caps.
            painter.line_segment(
                [
                    egui::pos2(rect.left(), mid_y),
                    egui::pos2(rect.right(), mid_y),
                ],
                egui::Stroke { width: stroke_w, color: white },
            );
            painter.line_segment(
                [
                    egui::pos2(rect.left(), mid_y - cap_half),
                    egui::pos2(rect.left(), mid_y + cap_half),
                ],
                egui::Stroke { width: stroke_w, color: white },
            );
            painter.line_segment(
                [
                    egui::pos2(rect.right(), mid_y - cap_half),
                    egui::pos2(rect.right(), mid_y + cap_half),
                ],
                egui::Stroke { width: stroke_w, color: white },
            );

            // ---- Inline text + BOX label, each with a drop-shadow ---------------
            ui.spacing_mut().item_spacing.y = 2.0;
            let text_shadow = egui::Color32::from_black_alpha(178);

            let meas = format!("{:.1} a\u{2080} \u{00b7} {:.3} nm", bar_a0, bar_nm);
            let box_txt = format!("BOX \u{00b1}{:.1} a\u{2080}", box_half);

            let body_font = egui::FontId::proportional(BODY_SIZE);
            let label_font = egui::FontId::proportional(LABEL_SIZE);

            // Allocate row for measurement text + paint shadow then primary.
            let meas_galley = ui.painter().layout_no_wrap(
                meas.clone(),
                body_font.clone(),
                TEXT_PRIMARY,
            );
            let (meas_rect, _) = ui.allocate_exact_size(
                meas_galley.size(),
                egui::Sense::hover(),
            );
            let painter = ui.painter();
            painter.text(
                meas_rect.left_top() + egui::vec2(1.0, 1.0),
                egui::Align2::LEFT_TOP,
                &meas,
                body_font.clone(),
                text_shadow,
            );
            painter.text(
                meas_rect.left_top(),
                egui::Align2::LEFT_TOP,
                &meas,
                body_font,
                TEXT_PRIMARY,
            );

            // BOX small-cap label: TEXT_TERTIARY at LABEL_SIZE, with shadow.
            let box_galley = ui.painter().layout_no_wrap(
                box_txt.clone(),
                label_font.clone(),
                TEXT_TERTIARY,
            );
            let (box_rect, _) = ui.allocate_exact_size(
                box_galley.size(),
                egui::Sense::hover(),
            );
            let painter = ui.painter();
            painter.text(
                box_rect.left_top() + egui::vec2(1.0, 1.0),
                egui::Align2::LEFT_TOP,
                &box_txt,
                label_font.clone(),
                text_shadow,
            );
            painter.text(
                box_rect.left_top(),
                egui::Align2::LEFT_TOP,
                &box_txt,
                label_font,
                TEXT_TERTIARY,
            );
        });
}

/// Renders the top-right 28x28 eye toggle. Clicking flips `hud_visible`.
/// When the HUD is hidden the widget dims to ~35 % opacity and swaps glyphs.
pub fn eye_toggle(ctx: &egui::Context, hud_visible: &mut bool) {
    egui::Area::new(egui::Id::new("eye-toggle"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-EDGE_INSET, EDGE_INSET))
        .show(ctx, |ui| {
            if !*hud_visible {
                ui.set_opacity(0.35);
            }
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(EYE_W, EYE_W),
                egui::Sense::click(),
            );
            let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

            let painter = ui.painter();
            painter.rect(
                rect,
                6.0,
                CARD_BG_LIGHT,
                egui::Stroke::new(1.0, BORDER),
                egui::StrokeKind::Inside,
            );

            let (glyph, color) = if *hud_visible {
                ("\u{25c9}", TEXT_SECONDARY)
            } else {
                ("\u{25ce}", TEXT_TERTIARY)
            };
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                glyph,
                egui::FontId::proportional(BODY_SIZE + 4.0),
                color,
            );

            if response.clicked() {
                *hud_visible = !*hud_visible;
            }
        });
}
