# Modern HUD redesign

Status: design

## Problem

The current HUD looks like a 1990s desktop app: default egui chrome (bevelled buttons, gray opaque side panel, beveled combo boxes), a fixed-width left panel that hogs space regardless of window aspect ratio, no visual hierarchy, no way to declutter for clean screenshots. Three readouts (FPS, peak |ψ|², scale bar) are scattered to corners without typographic relationship to each other.

## Goals

- Modern dark-glass aesthetic that complements the orbital render instead of fighting it.
- Adapts gracefully across aspect ratios (ultrawide → portrait).
- Lets the user hide individual surfaces or all overlays for clean screenshots.
- Discoverable controls — clicking a chip beats reading a slider label.

## Non-goals

- True backdrop-filter blur. egui+wgpu can't do it cheaply; alpha-only translucency suffices.
- Drag-anywhere card placement. Card snaps to a corner; no free-floating.
- Animation framework. Subtle hover state changes only; no eased transitions.
- Touch / mobile input. Mouse + keyboard only.
- Replacing the preset list. Stays at 11 named orbitals.

## Visual system

| Token | Value |
|---|---|
| `--card-bg` | `rgba(14, 12, 18, 0.85)` — was 0.74 in mockups; bumped because no blur |
| `--accent` | `#ff8a4c` (orange — does not track colormap) |
| `--accent-dim` | `rgba(255,138,76,0.30)` |
| `--text-primary` | `rgba(255,255,255,0.85)` |
| `--text-secondary` | `rgba(255,255,255,0.55)` |
| `--text-tertiary` | `rgba(255,255,255,0.35)` (labels, hints) |
| `--surface-mute` | `rgba(255,255,255,0.05)` (chip / button rest) |
| `--border` | `rgba(255,255,255,0.06)` |
| corner radius | 10 px card, 4 px chip, 999 px pill |
| label type | 9 px uppercase, 0.16em tracking |
| body type | 12 px regular |
| numerics | tabular figures |

## Layout

Single floating card anchored to the **top-left** corner (16 px inset), plus independent corner readouts. The card never docks.

```
┌─────────────────────────────────────────────────────────────┐
│ ┌────────────┐                  ╭── FPS pill ──╮  [eye/H]   │
│ │ Card       │                                              │
│ │ (title +   │            orbital                           │
│ │  groups)   │                                              │
│ └────────────┘                                              │
│ ─── scale 5.2 a₀ · 0.27 nm   box ±12 a₀                     │
│                                       [1s][2pz][3dz²]…More… │
└─────────────────────────────────────────────────────────────┘
```

### Adaptive behavior

The card is `min(320 px, 38vw)` wide. When the viewport width falls below 600 px the card auto-collapses to its title-only state (per **Card states** below). The preset chip strip wraps; once it would overlap the scale readout, the tail collapses behind a `More…` chip that opens a popover.

### Card states

1. **Expanded** (default) — title + all groups visible.
2. **Collapsed** — title only; chevron flips to `+`. Click to expand.
3. **Hidden-all** — card, pills, scale, presets all suppressed. Only the eye toggle remains, dimmed. Triggered by clicking the eye or pressing `H`.

State is in-memory only — defaults to **expanded** on every startup.

## Components

All components are custom-painted using egui's `Painter` API inside `egui::Area`. egui's built-in widgets (`Slider`, `ComboBox`, `Button`, `SidePanel`) are not used in the new HUD — they're the source of the 1990s look.

### Card

`egui::Area` anchored top-left. Frame:

- rounded rect, `--card-bg` fill, 1 px `--border` stroke
- title block (no separator, no background tint): `ATOM` 11 px bold uppercase + orbital name 16 px light + collapse chevron right-aligned
- groups separated by 1 px hairlines at `--border`

### Chip strip (used for n, l, m, grid resolution)

Custom widget. Renders a row of `Chip` rectangles:

- rest: `--surface-mute`, `--text-primary`
- selected: `--accent-dim` fill + 1 px `--accent` inset stroke
- disabled (invalid combo): same as rest but with 25 % alpha
- hover: lighten fill by ~+0.04 alpha

For `l` chips, label is `s p d f g h` (spectroscopic), not the integer.  
For `m` chips, label is signed integer (`−2`, `−1`, `0`, `+1`, `+2`).  
For grid chips, label is `128³` / `256³`.

Clicks on a disabled chip are no-ops (no error feedback needed — the constraint is self-documenting).

### Colormap swatches

Five horizontal strips, each painted with the actual colormap LUT (sampled from `colormaps::ALL`). Selected swatch shows a 1 px white inset + 1 px `--accent` outer ring. Hover: nothing (selection is unambiguous from the swatch itself).

### Slider (k, exposure)

Custom widget:

- track: 3 px, `--surface-mute`
- fill: `--accent-dim` from 0 to current value
- knob: 8 px circle, `--accent`, 6 px outer glow (`--accent-dim`)
- value: tabular numeric, right-aligned, `--text-primary`

Drag horizontally to scrub; click on the track jumps the knob.

### Toggle switch (auto-rotate)

Custom widget:

- off: 24×14 px pill, `--surface-mute`, white circle inside
- on: same pill in `--accent-dim`, circle slides to right, fills with `--accent`

### Action buttons (fit, capture)

Custom widget:

- rest: `--surface-mute` fill, `--text-primary`
- hover: alpha +0.04
- pressed: `--accent-dim` fill, 1 px `--accent` stroke
- shortcut hint (`F`, `S`) appended in `--text-tertiary` at smaller size

### HUD pill (FPS + peak |ψ|²)

`egui::Area` anchored top-right (inset to leave room for the eye toggle).

```
● 60.2 FPS │ peak |ψ|² 1.2e−3
```

- background: `rgba(14, 12, 18, 0.6)`
- 999 px corner radius
- 6 px accent dot left of the FPS number, with subtle outer glow

### Eye toggle (top-right corner, 16 px inset)

`egui::Area` anchored top-right, 28×28 px rounded square. Shows the eye glyph when HUD is visible (`◉`), the empty-eye glyph when hidden (`◎`). Tooltip: `Toggle HUD (H)`. Always visible.

### Scale readout (bottom-left)

Restyle of existing widget — same math as `ui.rs:138-196`, just new typography:

- 120 px bar (slightly narrower than 200 px), 1.5 px white stroke, white end caps
- inline text: `5.2 a₀ · 0.27 nm` + small-cap label `BOX ±12 a₀`
- text-shadow on this readout (it has no card behind it) so it stays legible over bright orbital lobes

### Preset chips (bottom-right)

Same chip widget as quantum numbers, but pill-shaped (999 px radius). A preset is highlighted iff its `(n, l, m)` exactly matches the current state — scrubbing the chip strips above can incidentally hit a preset and the highlight should follow. Clicking a chip sets `n, l, m`. If the strip would overlap the scale readout, the tail collapses into a `More…` chip that opens a popover (`egui::popup` family — exact API verified during implementation) listing the rest.

The 11 preset labels stay as defined in `ui.rs:35-47`.

## Keyboard

| Key | Action |
|---|---|
| `F` | Fit camera (existing) |
| `S` | Screenshot (existing) |
| `H` | Toggle hide-all-HUD (new) |
| `Space` | Toggle auto-rotate (new) |

All shortcuts work only when egui doesn't have keyboard focus (i.e., not while a text field is active — which the new HUD never has, but future-proofing).

## File / module changes

- `src/ui.rs` — rewrite. Drops `SidePanel`, `Slider`, `ComboBox`, `Button` usage. Becomes a thin orchestrator that places `Area`s and calls the new widget functions.
- `src/ui_widgets.rs` — **new**. One free function per custom widget: `chip_strip`, `swatch_row`, `glass_slider`, `toggle_switch`, `action_button`, `hud_pill`, `eye_toggle`, `preset_strip`, `scale_readout`, `card_frame`. Each takes `&mut egui::Ui` (or the `Painter` + a `Rect`) and the widget's state, returns a `Response`.
- `src/ui_tokens.rs` — **new**. Color and metric constants (the table above) as `Color32` / `f32` constants. Keeps theme tokens in one place.
- `src/main.rs` — wire `H` and `Space` shortcuts; pass viewport size to `ui::hud` so the adaptive breakpoint can fire.
- `src/colormaps.rs` — no change. The swatch widget samples the existing 256-entry LUT.
- `shaders/raymarch.wgsl` — no change.

`UiState` (`ui.rs:4-15`) gains two booleans: `hud_visible: bool` (default `true`) and `card_expanded: bool` (default `true`). Removes `colormap_index`'s combobox path but keeps the field.

## Implementation order

1. Token constants + card frame + glass slider — easiest visible win, validates the look.
2. Chip strip + swatch row — biggest interaction change.
3. Toggle, action buttons, HUD pill, eye toggle — small widgets, lots of them.
4. Adaptive breakpoint + collapse/hide-all state machine — wiring.
5. Scale readout + preset strip restyle.
6. Remove the old `SidePanel` body.

Each step should keep the app runnable; the new HUD lives alongside the old one until step 6.

## Out of scope

- Touch input or gesture support.
- True frosted-glass backdrop blur (deferred — would need a wgpu blur pass).
- Free drag of the card to any position (anchored to top-left only).
- Theme switching / light mode.
- Localization. All labels in English.
- Saving HUD state across sessions.
- New presets beyond the existing 11.

## Open risks

- **Custom widget hit-testing**: egui's input model expects widgets to allocate space, then check `Sense`. Painted custom widgets need careful `Rect` reuse between paint and sense passes; getting this wrong produces "looks right, doesn't click" bugs. Mitigation: write `chip_strip` first as the test case for the pattern.
- **Translucency over bright orbital lobes**: 0.85 alpha may still let the inferno colormap's brightest pixels bleed through and reduce contrast on text. If so, drop to 0.92 or add a 1-px-blur drop shadow under the card.
- **Adaptive breakpoint feel**: hard cutoff at 600 px width may feel abrupt during a window-resize drag. Smooth this only if it actually feels bad in practice — don't preemptively animate.
