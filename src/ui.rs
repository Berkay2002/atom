//! HUD control card: bounded (n, l, m) sliders inside a dark-glass card anchored
//! to the top-left of the viewport. Returns `true` from `panel()` if a parameter
//! that requires a volume rebake changed.

use crate::ui_tokens::{EDGE_INSET, LABEL_SIZE, TEXT_TERTIARY};
use crate::ui_widgets::{
    action_button, card_frame, chevron_button, chip_strip, eye_toggle, glass_slider, hud_pill,
    preset_strip, scale_readout, swatch_row, toggle_switch,
};

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
    /// Whether the HUD overlay (card, pill, scale bar) is visible. The eye
    /// toggle stays visible (dimmed) when this is `false`. In-memory only.
    pub hud_visible: bool,
    /// Whether the HUD card body (everything below the title row) is expanded.
    /// When `false` the card collapses to its title-only state. In-memory only.
    pub card_expanded: bool,
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
            hud_visible: true,
            card_expanded: true,
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
    if !s.hud_visible {
        return false;
    }
    let mut needs_rebake = false;
    let card_w = (0.38 * ctx.screen_rect().width()).min(320.0);
    egui::Area::new(egui::Id::new("hud-card"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(EDGE_INSET, EDGE_INSET))
        .show(ctx, |ui| {
            ui.set_width(card_w);
            card_frame(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("atom");
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            chevron_button(ui, &mut s.card_expanded);
                        },
                    );
                });
                if s.card_expanded {
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
                ui.label(
                    egui::RichText::new("COLORMAP")
                        .size(LABEL_SIZE)
                        .color(TEXT_TERTIARY),
                );
                if let Some(i) =
                    swatch_row(ui, crate::colormaps::ALL, s.colormap_index)
                {
                    s.colormap_index = i;
                }

                ui.label(
                    egui::RichText::new("RENDER")
                        .size(LABEL_SIZE)
                        .color(TEXT_TERTIARY),
                );
                glass_slider(ui, "k · sat.", &mut s.k, 0.1..=20.0, 1);
                glass_slider(ui, "exposure", &mut s.exposure, 0.1..=5.0, 1);
                // 512³ deferred per spec §3 decision 6 — sync bake on main thread
                // would freeze UI ~400ms; needs async + double-buffer first.
                let grid_labels = ["128\u{00b3}", "256\u{00b3}"];
                let grid_selected = if s.resolution == 128 { 0 } else { 1 };
                let grid_enabled = [true, true];
                if let Some(i) =
                    chip_strip(ui, &grid_labels, grid_selected, &grid_enabled, "grid")
                {
                    s.resolution = if i == 0 { 128 } else { 256 };
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("auto-rotate camera");
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            toggle_switch(ui, &mut s.auto_rotate);
                        },
                    );
                });
                ui.horizontal(|ui| {
                    // Split available width evenly between the two buttons.
                    let item_spacing = ui.spacing().item_spacing.x;
                    let half = (ui.available_width() - item_spacing) * 0.5;
                    ui.allocate_ui_with_layout(
                        egui::vec2(half, 0.0),
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
                            if action_button(ui, "fit", Some("F")).clicked() {
                                s.fit_requested = true;
                            }
                        },
                    );
                    ui.allocate_ui_with_layout(
                        egui::vec2(half, 0.0),
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
                            if action_button(ui, "capture", Some("S")).clicked() {
                                s.screenshot_requested = true;
                            }
                        },
                    );
                });

                if (s.n, s.l, s.m, s.resolution) != old {
                    needs_rebake = true;
                }
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

/// Renders the floating HUD overlays (eye toggle, pill, scale readout, preset
/// strip). Returns `true` if a preset chip was clicked and the volume needs to
/// be rebaked. The eye toggle is always visible (dimmed when hidden) so users
/// can restore the HUD without the keyboard shortcut.
pub fn hud(ctx: &egui::Context, h: &HudInputs, s: &mut UiState) -> bool {
    eye_toggle(ctx, &mut s.hud_visible);
    if !s.hud_visible {
        return false;
    }
    hud_pill(ctx, h.fps, h.peak_psi_sq);
    scale_readout(ctx, h.camera_radius, h.box_half);
    preset_strip(ctx, PRESETS, &mut s.n, &mut s.l, &mut s.m)
}
