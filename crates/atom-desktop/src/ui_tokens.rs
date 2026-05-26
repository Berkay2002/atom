//! Design tokens for the modern HUD: colors and metrics from the PRD visual-system
//! table. No functions; this file is constants only.

use egui::Color32;

// ---- Colors --------------------------------------------------------------

/// Card background — rgba(14, 12, 18, 0.85)
pub const CARD_BG: Color32 = Color32::from_rgba_unmultiplied_const(14, 12, 18, 217);

/// Lighter card variant for floating pills — rgba(14, 12, 18, 0.60).
pub const CARD_BG_LIGHT: Color32 = Color32::from_rgba_unmultiplied_const(14, 12, 18, 153);

/// Accent — #ff8a4c
pub const ACCENT: Color32 = Color32::from_rgb(255, 138, 76);

/// Dimmed accent — rgba(255, 138, 76, 0.30)
pub const ACCENT_DIM: Color32 = Color32::from_rgba_unmultiplied_const(255, 138, 76, 76);

/// Primary text — rgba(255, 255, 255, 0.85)
pub const TEXT_PRIMARY: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 217);

/// Secondary text — rgba(255, 255, 255, 0.55)
pub const TEXT_SECONDARY: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 140);

/// Tertiary text — rgba(255, 255, 255, 0.35)
pub const TEXT_TERTIARY: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 89);

/// Muted surface fill — rgba(255, 255, 255, 0.05)
pub const SURFACE_MUTE: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 13);

/// Hairline border — rgba(255, 255, 255, 0.06)
pub const BORDER: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 15);

// ---- Metrics -------------------------------------------------------------

/// Corner radius for card-sized surfaces.
pub const RADIUS_CARD: f32 = 10.0;

/// Corner radius for chip-sized surfaces.
pub const RADIUS_CHIP: f32 = 4.0;

/// Corner radius for fully-rounded pills (egui clamps).
pub const RADIUS_PILL: f32 = 999.0;

/// Label text size in pixels (uppercase, 0.16em tracking — applied at call site).
pub const LABEL_SIZE: f32 = 9.0;

/// Body text size in pixels.
pub const BODY_SIZE: f32 = 12.0;

/// Canvas-edge inset for HUD cards.
pub const EDGE_INSET: f32 = 16.0;
