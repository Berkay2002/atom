# Atom

Real-time visualizer for atomic orbital density.

Atom renders the selected atom's `|psi_nlm|^2` density as a continuous 3D field.
The CPU bakes the wavefunction into a volume texture; the GPU ray-marches that
texture for an interactive glowing-cloud view. The same Rust math powers the
native desktop app and the browser demo.

## What It Shows

- Single-atom orbital density for elements H through Ar.
- Bounded quantum numbers `n`, `l`, and `m`.
- Real spherical-harmonic orbital shapes, including familiar `s`, `p`, `d`, and
  `f` orientations.
- Slater-screened effective nuclear charge, with a bare-Z comparison toggle.
- Multiple colormaps, exposure/saturation controls, camera orbit, auto-rotate,
  presets, and guided tours in the web app.

## Repository Layout

```text
crates/
  atom-core/      Shared Rust physics, scene encoding, element data, volume bake
  atom-desktop/   Native wgpu/winit/egui renderer
web/              Next.js + WebGL2 browser demo using atom-core via WASM
```

## Requirements

- Rust stable toolchain.
- A GPU and driver with `wgpu` support for the desktop app.
- Node.js 20.19+ for the web app.
- `wasm-pack` for building the web app's WASM bundle.

Install `wasm-pack` if needed:

```powershell
cargo install wasm-pack
```

## Run The Desktop App

From the repo root:

```powershell
cargo run --release -p atom-desktop
```

Use a release build for normal runs. Debug builds make volume bakes much slower.

Desktop controls include element and quantum-number pickers, effective-Z/bare-Z
toggle, colormap selection, render controls, fit, auto-rotate, and screenshot
capture. Captured screenshots are written as `orbital_*.png` in the repo root.

## Run The Web App

From the repo root:

```powershell
cd web
npm install
npm run dev
```

`npm run dev` rebuilds the WASM bundle before starting Next.js. If Rust sources
change while the dev server is already running, rebuild WASM with:

```powershell
npm run wasm
```

Then refresh the browser.

## Test

Rust workspace:

```powershell
cargo test
```

Web app:

```powershell
cd web
npm test
npm run lint
```

## Build

Desktop release binary:

```powershell
cargo build --release -p atom-desktop
```

Web production build:

```powershell
cd web
npm run build
```

## Notes For Contributors

- Keep the wavefunction math in `atom-core`; the desktop and web targets should
  share it rather than reimplementing it separately.
- Run cargo commands from the repo root unless you are working inside `web/`.
- Keep commits conventional, for example `feat:`, `fix:`, `test:`, `chore:`, or
  `refactor:`.
