//! HUD control card: bounded (n, l, m) sliders inside a dark-glass card anchored
//! to the top-left of the viewport. Returns `true` from `panel()` if a parameter
//! that requires a volume rebake changed.

use atom_core::scene::{Atom, ElementId, Orbital, Scene, View};

use crate::ui_tokens::{BODY_SIZE, EDGE_INSET, LABEL_SIZE, TEXT_SECONDARY, TEXT_TERTIARY};
use crate::ui_widgets::{
    action_button, card_frame, chevron_button, chip_strip, eye_toggle, glass_slider, hud_pill,
    preset_strip, scale_readout, swatch_row, toggle_switch,
};

pub struct UiState {
    /// Atomic number (1..=18) of the selected element. Drives the per-atom
    /// `z_eff` resolution in `bake_scene`. Issue 02 ships a functional but
    /// unstyled picker; issue 05 will polish it into a periodic-table grid.
    pub element_z: u32,
    pub n: u32,
    pub l: u32,
    pub m: i32,
    /// When `true`, the bake uses the bare atomic number `Z` instead of
    /// the Slater-screened `z_eff`. Lets users watch shielding's effect
    /// on orbital size by comparison. Threaded through to
    /// `View.use_bare_z` in `main.rs` so URL serialization (issue 04)
    /// captures it automatically.
    pub use_bare_z: bool,
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
    /// `true` when the adaptive breakpoint (< 600 px viewport) has forced the
    /// card into its collapsed state. Cleared when the viewport widens again,
    /// or when the user manually clicks the chevron while narrow (manual
    /// override wins).
    pub auto_collapsed: bool,
    /// Previous-frame narrow state. Used to detect the wide→narrow edge so we
    /// only auto-collapse once per crossing — otherwise a manual expand while
    /// still narrow would be undone on the next frame.
    pub prev_narrow: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            element_z: 1,
            n: 3,
            l: 2,
            m: 1,
            use_bare_z: false,
            resolution: 256,
            k: 5.0,
            exposure: 1.0,
            auto_rotate: false,
            colormap_index: 0,
            fit_requested: false,
            screenshot_requested: false,
            hud_visible: true,
            card_expanded: true,
            auto_collapsed: false,
            prev_narrow: false,
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
    let viewport_w = ctx.screen_rect().width();
    let narrow = viewport_w < 600.0;
    // Track breakpoint crossings on the edge only. On wide→narrow we auto-
    // collapse if the user had the card expanded. On narrow→wide we release
    // the flag so the user's last manual intent (`card_expanded`) takes over
    // again. Sampling only on edges means a manual chevron click that clears
    // `auto_collapsed` while still narrow won't be re-asserted next frame.
    if narrow && !s.prev_narrow && s.card_expanded {
        s.auto_collapsed = true;
    } else if !narrow && s.prev_narrow {
        s.auto_collapsed = false;
    }
    s.prev_narrow = narrow;
    let card_w = (0.38 * viewport_w).min(320.0);
    egui::Area::new(egui::Id::new("hud-card"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(EDGE_INSET, EDGE_INSET))
        .show(ctx, |ui| {
            ui.set_width(card_w);
            card_frame(ui, |ui| {
                ui.horizontal(|ui| {
                    // Right-to-left layout over the FULL row width: chevron
                    // gets placed first at the right edge, then the heading
                    // fills the remaining space on the left.
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            // A manual chevron click is the user's explicit
                            // intent, so it releases any breakpoint-driven
                            // auto-collapse. This lets the user re-open the
                            // card while the viewport is still narrow.
                            if chevron_button(ui, &mut s.card_expanded).clicked() {
                                s.auto_collapsed = false;
                            }
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.heading("atom");
                                },
                            );
                        },
                    );
                });
                let render_expanded = s.card_expanded && !s.auto_collapsed;
                if render_expanded {
                let old = (s.element_z, s.n, s.l, s.m, s.use_bare_z, s.resolution);

                // "What am I looking at" caption (issue 06). Sits at the top
                // of the card so the visitor immediately reads the plain-
                // language name of the current orbital before they touch any
                // chip. Built from atom-core::caption so the desktop and
                // web targets stay byte-identical on the string they show.
                let scene = Scene {
                    atoms: vec![Atom {
                        element: ElementId(s.element_z),
                        position: [0.0, 0.0, 0.0],
                        orbital: Orbital { n: s.n, l: s.l, m: s.m },
                    }],
                    view: View::default(),
                };
                let caption = atom_core::caption(&scene);
                ui.label(
                    egui::RichText::new(caption)
                        .size(BODY_SIZE)
                        .color(TEXT_SECONDARY),
                );
                ui.add_space(6.0);

                // Element picker — functional, unstyled per issue 02.
                // 18 chips laid out in three 6-wide rows so each chip
                // stays clickable at the card's 320px width. Issue 05
                // will replace this with a polished periodic-table grid.
                ui.label(
                    egui::RichText::new("ELEMENT")
                        .size(LABEL_SIZE)
                        .color(TEXT_TERTIARY),
                );
                const ELEMENT_LABELS: [&str; 18] = [
                    "H", "He", "Li", "Be", "B", "C",
                    "N", "O", "F", "Ne", "Na", "Mg",
                    "Al", "Si", "P", "S", "Cl", "Ar",
                ];
                let element_selected = (s.element_z as usize).saturating_sub(1).min(17);
                for row in 0..3 {
                    let lo = row * 6;
                    let hi = lo + 6;
                    let labels: &[&str] = &ELEMENT_LABELS[lo..hi];
                    let enabled = [true; 6];
                    let sel_in_row = if element_selected >= lo && element_selected < hi {
                        element_selected - lo
                    } else {
                        usize::MAX // not in this row → render none selected
                    };
                    let salt = format!("element-row-{row}");
                    if let Some(idx) = chip_strip(ui, labels, sel_in_row, &enabled, &salt) {
                        s.element_z = (lo + idx) as u32 + 1;
                        // (n, l, m) constraints are hydrogen-like and
                        // universal across elements (l < n, |m| <= l), so
                        // no element-driven re-clamp is needed here. The
                        // re-bake fires below via the `old` comparison.
                    }
                }

                // Bare-Z toggle — sits directly below the element picker so
                // the relationship between element choice and shielding is
                // visually grouped. Off (default) uses Slater-screened
                // `z_eff`; on uses the bare atomic number `Z` and the
                // orbital collapses inward (e.g. carbon's 2p shrinks).
                ui.horizontal(|ui| {
                    let label = if s.use_bare_z { "bare Z" } else { "effective Z" };
                    ui.label(label);
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            toggle_switch(ui, &mut s.use_bare_z);
                        },
                    );
                })
                .response
                .on_hover_text(
                    "Bare Z removes electron shielding to show what the orbital \
                     would look like if the nucleus's full charge reached the electron.",
                );

                ui.separator();

                // Group the QUANTUM NUMBERS label + three chip rows with a
                // slightly larger vertical rhythm so the rows breathe.
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.y = 5.0;
                    ui.label(
                        egui::RichText::new("QUANTUM NUMBERS")
                            .size(LABEL_SIZE)
                            .color(TEXT_TERTIARY),
                    );

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
                });

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
                glass_slider(ui, "saturation", &mut s.k, 0.1..=20.0, 1);
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
                // `ui.columns` enforces strictly equal column widths, so the
                // two action buttons are guaranteed to render at identical
                // size regardless of label length.
                ui.columns(2, |cols| {
                    if action_button(&mut cols[0], "fit", Some("F")).clicked() {
                        s.fit_requested = true;
                    }
                    if action_button(&mut cols[1], "capture", Some("S")).clicked() {
                        s.screenshot_requested = true;
                    }
                });

                if (s.element_z, s.n, s.l, s.m, s.use_bare_z, s.resolution) != old {
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
