# AGENTS.md

## Project

`atom` — real-time, interactive 3D visualizer of hydrogen `|ψ_nlm|²` probability density. Rust + wgpu + winit + egui. CPU bakes the wavefunction into a 3D texture (rayon over voxels); GPU fragment shader ray-marches it.

## Build, run, test

- **Run the app:** `cargo run --release`. Debug builds make the volume bake 5–10× slower — don't propose `cargo run` for performance work.
- **Tests:** `cargo test`. Only `physics.rs` has `#[test]` blocks — the wavefunction math is verified against scipy. Renderer, volume bake, camera, and UI are verified visually; **do not invent unit tests for those modules** (mocking wgpu/egui is more work than it's worth and doesn't catch real bugs).
- Run cargo from the project root (`C:\Users\berka\.me\projects\atom`).

## Module responsibilities

One responsibility per file:

- `src/physics.rs` — Laguerre, Legendre, real `Y_lm`, `|ψ|²` evaluator. Only file with `#[test]`.
- `src/volume.rs` — adaptive box size, rayon voxel bake, peak normalization.
- `src/render.rs` — 3D texture upload, ray-march pipeline, colormap 1D LUT textures.
- `src/camera.rs` — orbit camera, view/proj matrices.
- `src/ui.rs` — egui side panel, screenshot dispatch, preset mapping.
- `src/colormaps.rs` — 256-entry RGB LUT constants.
- `src/main.rs` — winit ApplicationHandler, wgpu init, frame loop.
- `shaders/raymarch.wgsl` — full-screen-triangle vertex + ray-march fragment.

## Conventions

- **Commits:** Conventional Commits (`feat:`, `fix:`, `test:`, `chore:`, `refactor:`).
- **Shell:** PowerShell 7 on Windows — pipeline chain `&&`/`||`, `$env:VAR`, here-strings `@'...'@`. Don't fall back to bash idioms.
- **Screenshots** the app captures land at `orbital_*.png` in the repo root and are gitignored. The screenshot path captures only the orbital, not the egui overlay.
- Reference C++ files (`atom_raytracer.cpp`, `atom_realtime.cpp`) are intent only — port to idiomatic Rust, do not transliterate.

## Agent skills

### Issue tracker

Issues live as markdown files under `.scratch/<feature>/` in this repo. See `docs/agents/issue-tracker.md`.

### Triage labels

Default canonical vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
