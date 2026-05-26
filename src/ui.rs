//! HUD control card: bounded (n, l, m) sliders inside a dark-glass card anchored
//! to the top-left of the viewport. Returns `true` from `panel()` if a parameter
//! that requires a volume rebake changed.

use crate::ui_tokens::{EDGE_INSET, LABEL_SIZE, TEXT_TERTIARY};
use crate::ui_widgets::{card_frame, chip_strip};

pub struct UiState {
    pub n: u32,
    pub l: u32,
    pub m: i32,
    pub resolution: usize,
    pub k: f32,
    pub exposure: f32,
    pub auto_rotate: bool,
    pub colormap_index: usize,
    pub fit_requested: bool,
    pub screenshot_requested: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            n: 3,
            l: 2,
            m: 1,
            resolution: 256,
            k: 5.0,
            exposure: 1.0,
            auto_rotate: false,
            colormap_index: 0,
            fit_requested: false,
            screenshot_requested: false,
        }
    }
}

/// Named (label, n, l, m) presets for the quick-jump dropdown.
pub const PRESETS: &[(&str, u32, u32, i32)] = &[
    ("1s",            1, 0,  0),
    ("2s",            2, 0,  0),
    ("2p_x",          2, 1,  1),
    ("2p_y",          2, 1, -1),
    ("2p_z",          2, 1,  0),
    ("3d_xy",         3, 2, -2),
    ("3d_xz",         3, 2,  1),
    ("3d_yz",         3, 2, -1),
    ("3d_(x^2-y^2)",  3, 2,  2),
    ("3d_(z^2)",      3, 2,  0),
    ("4f_(z^3)",      4, 3,  0),
];

pub fn panel(ctx: &egui::Context, s: &mut UiState) -> bool {
    let mut needs_rebake = false;
    let card_w = (0.38 * ctx.screen_rect().width()).min(320.0);
    egui::Area::new(egui::Id::new("hud-card"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(EDGE_INSET, EDGE_INSET))
        .show(ctx, |ui| {
            ui.set_width(card_w);
            card_frame(ui, |ui| {
                ui.heading("atom");
                let mut preset_choice: Option<usize> = None;
                egui::ComboBox::from_label("preset")
                    .selected_text("choose…")
                    .show_ui(ui, |ui| {
                        for (i, p) in PRESETS.iter().enumerate() {
                            if ui.selectable_label(false, p.0).clicked() {
                                preset_choice = Some(i);
                            }
                        }
                    });
                if let Some(i) = preset_choice {
                    let p = PRESETS[i];
                    s.n = p.1;
                    s.l = p.2;
                    s.m = p.3;
                    needs_rebake = true;
                }
                ui.separator();
                ui.label(
                    egui::RichText::new("QUANTUM NUMBERS")
                        .size(LABEL_SIZE)
                        .color(TEXT_TERTIARY),
                );

                let old = (s.n, s.l, s.m, s.resolution);

                // n: 1..=6, always all enabled.
                let n_labels = ["1", "2", "3", "4", "5", "6"];
                let n_enabled = [true; 6];
                let n_selected = (s.n as usize).saturating_sub(1).min(5);
                if let Some(idx) =
                    chip_strip(ui, &n_labels, n_selected, &n_enabled, "qn-n")
                {
                    s.n = (idx as u32) + 1;
                    if s.l > s.n - 1 {
                        s.l = s.n - 1;
                    }
                    let l_i = s.l as i32;
                    s.m = s.m.clamp(-l_i, l_i);
                }

                // l: spectroscopic letters s,p,d,f,g,h (positions 0..=5).
                // Enabled where position <= s.n - 1.
                let l_labels = ["s", "p", "d", "f", "g", "h"];
                let n_minus_1 = (s.n as usize).saturating_sub(1);
                let l_enabled: [bool; 6] = std::array::from_fn(|i| i <= n_minus_1);
                let l_selected = (s.l as usize).min(5);
                if let Some(idx) =
                    chip_strip(ui, &l_labels, l_selected, &l_enabled, "qn-l")
                {
                    s.l = idx as u32;
                    let l_i = s.l as i32;
                    s.m = s.m.clamp(-l_i, l_i);
                }

                // m: -5..=+5 always rendered; enabled where |position| <= s.l.
                let m_labels: [&str; 11] = [
                    "\u{2212}5", "\u{2212}4", "\u{2212}3", "\u{2212}2", "\u{2212}1",
                    "0",
                    "+1", "+2", "+3", "+4", "+5",
                ];
                let l_i = s.l as i32;
                let m_enabled: [bool; 11] =
                    std::array::from_fn(|i| (i as i32 - 5).abs() <= l_i);
                let m_selected = (s.m + 5).clamp(0, 10) as usize;
                if let Some(idx) =
                    chip_strip(ui, &m_labels, m_selected, &m_enabled, "qn-m")
                {
                    s.m = idx as i32 - 5;
                }

                ui.separator();
                ui.label("Visual");
                egui::ComboBox::from_label("resolution")
                    .selected_text(format!("{}^3", s.resolution))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut s.resolution, 128, "128^3");
                        ui.selectable_value(&mut s.resolution, 256, "256^3");
                        // 512^3 deferred per spec §3 decision 6 — sync bake on main thread
                        // would freeze UI ~400ms; needs async + double-buffer first.
                    });
                let cmap_names: Vec<&str> =
                    crate::colormaps::ALL.iter().map(|(n, _)| *n).collect();
                let current_name = cmap_names
                    .get(s.colormap_index)
                    .copied()
                    .unwrap_or("inferno");
                egui::ComboBox::from_label("colormap")
                    .selected_text(current_name)
                    .show_ui(ui, |ui| {
                        for (i, name) in cmap_names.iter().enumerate() {
                            ui.selectable_value(&mut s.colormap_index, i, *name);
                        }
                    });
                ui.add(egui::Slider::new(&mut s.k, 0.1..=20.0).text("k (saturation)"));
                ui.add(egui::Slider::new(&mut s.exposure, 0.1..=5.0).text("exposure"));
                ui.checkbox(&mut s.auto_rotate, "auto-rotate camera");

                ui.separator();
                if ui.button("Fit camera (F)").clicked() {
                    s.fit_requested = true;
                }
                if ui.button("Screenshot (S)").clicked() {
                    s.screenshot_requested = true;
                }

                if (s.n, s.l, s.m, s.resolution) != old {
                    needs_rebake = true;
                }
            });
        });
    needs_rebake
}

pub struct HudInputs {
    pub fps: f32,
    pub peak_psi_sq: f64,    // a₀^-3
    pub box_half: f64,       // a₀ (current orbital)
    pub camera_radius: f32,  // a₀
}

pub fn hud(ctx: &egui::Context, h: &HudInputs) {
    egui::Area::new(egui::Id::new("fps"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
        .show(ctx, |ui| {
            ui.label(format!("{:5.1} FPS", h.fps));
        });
    egui::Area::new(egui::Id::new("density"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-12.0, -12.0))
        .show(ctx, |ui| {
            ui.label(format!("peak |ψ|² = {:.3e} a₀⁻³", h.peak_psi_sq));
        });
    egui::Area::new(egui::Id::new("scale"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(12.0, -12.0))
        .show(ctx, |ui| {
            // Visible scale bar: fixed 200 px wide. Compute world width that maps to
            // 200 px at the current camera distance using a perspective-fov approximation:
            //   visible_world_half = camera_radius · tan(fov/2)
            // We use fov=60° (matches Camera::new), aspect via screen size.
            let bar_px = 200.0_f32;
            let viewport_h = ctx.screen_rect().height();
            // visible_world_h spans the FULL window height at z = camera_radius.
            let visible_world_h = 2.0 * h.camera_radius * (60.0_f32.to_radians() * 0.5).tan();
            let a0_per_px = visible_world_h / viewport_h;
            let bar_a0 = (bar_px * a0_per_px) as f64;
            let bar_nm = bar_a0 * 0.052_917_7;
            let (response, painter) = ui.allocate_painter(
                egui::vec2(bar_px, 18.0),
                egui::Sense::hover(),
            );
            let rect = response.rect;
            let mid_y = rect.center().y;
            let color = egui::Color32::WHITE;
            painter.line_segment(
                [
                    egui::pos2(rect.left(), mid_y),
                    egui::pos2(rect.right(), mid_y),
                ],
                egui::Stroke { width: 2.0, color },
            );
            // Small end caps.
            painter.line_segment(
                [
                    egui::pos2(rect.left(), mid_y - 5.0),
                    egui::pos2(rect.left(), mid_y + 5.0),
                ],
                egui::Stroke { width: 2.0, color },
            );
            painter.line_segment(
                [
                    egui::pos2(rect.right(), mid_y - 5.0),
                    egui::pos2(rect.right(), mid_y + 5.0),
                ],
                egui::Stroke { width: 2.0, color },
            );
            ui.label(format!(
                "{:.1} a₀  ({:.3} nm)   |   box: ±{:.1} a₀",
                bar_a0, bar_nm, h.box_half
            ));
        });
}
