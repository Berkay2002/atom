# Spec: Real-time hydrogen orbital visualizer (Rust + wgpu)

Single source of truth. Captures both the validated physics and every design decision made for this project.

Target machine: Windows 11, RTX 3090 (24 GB VRAM), Ryzen 7 7800X3D (8C/16T, 96 MB L3).

Reference C++/OpenGL code is in `atom_raytracer.cpp` and `atom_realtime.cpp`. Treat as **intent only** — they capture the feel of the original idea. Port to idiomatic Rust; do not transliterate. Where the C++ disagrees with anything in this spec, this spec wins.

---

## 1. What it is

Interactive 3D visualizer of the hydrogen `|ψ_nlm|²` probability density. User picks `(n, l, m)` and a few visual knobs from a side panel; the volume is baked on CPU and ray-marched on GPU.

No Monte-Carlo sampling, no BVH, no spheres — the volume *is* the density. Continuous glowing-fog look, not a particle cloud.

## 2. Physics — validated, treat as ground truth

The math below was verified in an earlier session against `scipy` (`genlaguerre` + `sph_harm`) to ~1e-18 absolute error in `|ψ|²` over 3200 random samples across 16 `(n,l,m)` cases; Monte-Carlo normalization integrals came out ≈ 1.0. Use these exact recurrences; do not re-derive.

Units: Bohr radius `a₀ = 1`, `Z = 1`. Wavefunction `ψ_nlm(r,θ,φ) = R_nl(r) · Y_lm(θ,φ)`, so `|ψ|² = R_nl(r)² · Y_lm(θ,φ)²`.

### 2.1 Radial part

```
ρ = 2r/n
R_nl(r) = sqrt( (2/n)^3 · (n-l-1)! / (2n·(n+l)!) ) · e^(-ρ/2) · ρ^l · L_{n-l-1}^{2l+1}(ρ)
```

### 2.2 Associated Laguerre `L_p^q(x)`, with `p = n-l-1`, `q = 2l+1`

Stable upward recurrence:
```
L_0 = 1
L_1 = 1 + q - x
(k+1)·L_{k+1} = (2k+1+q-x)·L_k - (k+q)·L_{k-1}
```

### 2.3 Associated Legendre `P_l^m(x)`, `m ≥ 0`, `x = cosθ`

Condon–Shortley sign included:
```
pmm = 1
for i in 1..=m:  pmm *= -(2i-1) * sqrt(1 - x*x)
P_m^m     = pmm                                              # base case l == m
P_{m+1}^m = x*(2m+1)*pmm                                     # base case l == m+1
P_l^m     = ((2l-1)*x*P_{l-1}^m - (l+m-1)*P_{l-2}^m) / (l-m) # for l >= m+2
```

### 2.4 Real spherical harmonics

We use **real** spherical harmonics, not complex. This is a deliberate choice (see §3, decision 1): real `Y_lm` produces the chemistry-textbook lobed orbitals (`p_x`, `d_xy`, etc.) and makes the sign of `m` visually meaningful.

```
K = sqrt( (2l+1)/(4π) · (l-|m|)! / (l+|m|)! )
m > 0:  Y = sqrt(2) · K · cos(m·φ)   · P_l^m(cosθ)
m = 0:  Y =           K ·              P_l^0(cosθ)
m < 0:  Y = sqrt(2) · K · sin(|m|·φ) · P_l^|m|(cosθ)
```

---

## 3. Design decisions (with rationale)

Each decision was made deliberately; record the why so future-you doesn't quietly reverse one.

1. **Real spherical harmonics, not complex.** Real basis gives recognizable lobed shapes and makes every `(l, m)` visually distinct; complex collapses `±m` into identical donuts. Both are equally physically valid eigenstates of the hydrogen Hamiltonian.
2. **GPU volume ray-march, not point sampling.** Render the actual `|ψ|²` field, not a Monte-Carlo approximation. Cost decouples from any user-tunable sample count. Honors the "ray tracing" word in the original brief more honestly than rasterized impostor billboards do. No BVH, no spheres, no CDF inverters anywhere.
3. **Per-orbital adaptive box, fixed-resolution voxel grid, world-scale camera.** Box edge length `= 6·n²·a₀`, scaled to fit each orbital. Voxel count constant → angular detail constant across `n`. Camera does **not** auto-zoom on `n` change, so `n=1` looks small and `n=6` looks huge on screen — preserves the real-physics fact that orbital size scales as `n²`. User has mouse-wheel zoom and an `F`-key "fit to current orbital" snap.
4. **Emission-only ray accumulation with saturation.** Per pixel: `sum = ∫ |ψ|² ds` along ray, then `intensity = 1 - exp(-k · sum)`. Avoids the front-face brightness bias of alpha compositing — symmetric orbitals render symmetric. Saturation prevents bright cores blowing out. `k` is a live slider.
5. **Per-orbital normalize-to-peak, with absolute peak shown in HUD.** Each baked volume is divided by its own peak so the render input is always in `[0, 1]` and a single global `k` works across all orbitals. The absolute peak `|ψ|²` value (e.g. `3.2e-5 a₀⁻³`) is displayed in the HUD so the ~10⁴× density spread across `n` is not hidden, just not visualized.
6. **256³ voxels default, sync rebake on main thread.** 7800X3D + rayon bakes 256³ in ~50 ms. Tolerable single-frame stutter on a discrete `(n,l,m)` change. egui dropdown allows 128/256/512; only 512 would need async + double-buffer, deferred.
7. **egui side panel for all controls.** Live sliders for tuning knobs are essential; pure-keyboard UI would force a recompile-and-eyeball loop on every `k`/exposure tweak. Version-pinning egui/egui-wgpu/wgpu/winit is the main pain point; pin in `Cargo.lock` and don't update mid-project.
8. **Range-bounded `(n, l, m)` sliders.** `l` slider's range is `0..n-1` and follows `n`; `m` slider's range is `-l..+l` and follows `l`. Invalid quantum-number states are unreachable by construction. Preset dropdown (1s, 2p_z, …) for quick jumps.
9. **Multiple colormaps, live-switchable.** Six baked 256-pixel 1D LUT textures: inferno (default), viridis, magma, plasma, grayscale, electron-blue. Switching = swap texture binding. Tables sourced from matplotlib (public domain).
10. **Auto-rotate camera, not animated density.** Default off, toggle in egui, 0.2 rad/s when on. Replaces the C++ reference's swirling `calculateProbabilityFlow` — that animation lies for real SH (where the true probability current is identically zero). Camera rotation gives motion + 3D parallax without forging dynamics.

---

## 4. Volume bake

- **Resolution**: 256³ default. egui dropdown for 128 / 256 / 512.
- **Box**: cubic, centered on nucleus, edge length `6·n²·a₀`. Snug fit.
- **Bake**: rayon `par_iter` over voxels, evaluate `|ψ|²` at voxel center. ~50 ms wall-time for 256³ on 7800X3D.
- **Storage**: GPU `texture_3d<f32>`, single-channel `R32Float`. ~64 MB at 256³, ~512 MB at 512³.
- **Normalization**: find peak `|ψ|²` in the baked grid; divide all voxels by peak. Store the absolute peak value for HUD display. Render input ∈ [0, 1].
- **Rebake trigger**: any change to `(n, l, m)` or resolution. `k`/exposure/colormap/camera are shader uniforms only — no rebake.

---

## 5. Render

GPU volume ray-march in a fragment shader (full-screen triangle).

**Per pixel:**
1. Compute ray origin/direction from camera, transform into box-local space.
2. Slab-intersect ray with `[-1, 1]³` box; if miss → background (`#000000`) and exit.
3. March from entry to exit in `N` fixed steps (`N` = current resolution).
4. At each step, sample 3D texture trilinearly. Accumulate `sum += density * step_length`.
5. Final intensity: `i = clamp(exposure * (1 - exp(-k * sum)), 0, 1)`.
6. Sample colormap 1D texture at `i`. Output RGB.

No alpha compositing. No order dependence. No early termination in v1 (add if 512³ ever shows perf issues).

---

## 6. Camera

Orbit camera in world space. Fixed world scale — does **not** auto-zoom on `n` change.

- Mouse drag: orbit (azimuth + elevation).
- Mouse scroll: zoom (change radius from origin).
- `F` key: snap to "fit current orbital" — radius = 2× current box half-extent, elevation 30°, azimuth 45°.
- Auto-rotate toggle in egui: continuous azimuth at 0.2 rad/s (~30 s/revolution). Default off.

Initial state on launch: `(n=3, l=2, m=1)` (a real `d_xz`), auto-rotate off, camera in fit position.

---

## 7. UI (egui + egui-wgpu)

Single left-side panel:

- **Quantum numbers**: integer sliders `n` (1–6), `l` (0..n−1), `m` (−l..+l). Ranges of `l` and `m` shrink/grow live with `n` and `l`.
- **Preset dropdown**: `1s`, `2s`, `2p_x`, `2p_y`, `2p_z`, `3d_xy`, `3d_xz`, `3d_yz`, `3d_{x²-y²}`, `3d_{z²}`, `4f_{z³}` — sets `(n,l,m)` in one click.
- **Visual**:
  - Resolution: 128 / 256 / 512.
  - Colormap: inferno / viridis / magma / plasma / grayscale / electron-blue.
  - `k` slider: 0.1 – 20.0, default 5.0.
  - Exposure slider: 0.1 – 5.0, default 1.0.
  - Auto-rotate checkbox.
- **Camera**: "Fit" button (mirrors `F` key).
- **Screenshot**: button + `S` hotkey → save `orbital_n{n}l{l}m{m}_{timestamp}.png` to current directory.

HUD corners:
- **Top-right**: FPS counter.
- **Bottom-left**: scale bar — line labeled `10 a₀ (0.53 nm)`, length matches world distance under current camera.
- **Bottom-right**: density readout — `peak |ψ|² = 3.2e-5 a₀⁻³`.

---

## 8. Stack

```toml
[dependencies]
wgpu       = "29"      # check 29.x exact API on docs.rs; request_adapter returns Result, request_device takes single descriptor
winit      = "0.30"    # ApplicationHandler trait, NOT the 0.29 closure API
egui       = "0.32"    # pin alongside egui-wgpu
egui-wgpu  = "0.32"    # must match egui + wgpu versions
pollster   = "0.4"
bytemuck   = { version = "1", features = ["derive"] }
glam       = "0.33"
rayon      = "1"
image      = "0.25"    # PNG export
```

Pin everything in `Cargo.lock`. Do not update wgpu/winit/egui mid-project. The egui ↔ egui-wgpu ↔ wgpu ↔ winit version quartet is the only real version-pinning pain point.

---

## 9. Layout

```
atom/
  Cargo.toml
  src/
    main.rs       # winit ApplicationHandler, wgpu init, frame loop, egui integration
    physics.rs    # Laguerre, Legendre, real Y_lm, |ψ|² evaluator
    volume.rs     # adaptive box, rayon voxel bake, peak normalization
    render.rs     # 3D texture upload, ray-march pipeline, colormap LUT textures
    camera.rs     # orbit + zoom + fit + auto-rotate
    ui.rs         # egui panel layout, screenshot dispatch
  shaders/
    raymarch.wgsl
  spec.md
  atom_raytracer.cpp   # C++/OpenGL reference (intent only)
  atom_realtime.cpp    # C++/OpenGL reference (intent only)
```

---

## 10. Build order

Verify each step before moving on. No skipping.

1. Cargo scaffold + deps. winit 0.30 `ApplicationHandler` opens a window, wgpu clears to black. Run, confirm.
2. `physics.rs`: implement Laguerre/Legendre recurrences and real `Y_lm`. Add `#[test]` cases:
   - Known closed-form values: `ψ_{1,0,0}(0,0,0) = 1/sqrt(π)`, so `|ψ|²(0,0,0) = 1/π`.
   - Two or three more sampled points spot-checked numerically.
   - Numerical integral `∫|ψ_nlm|² dV ≈ 1` for `(2,1,0)` and `(3,2,1)` on a 64³ grid.
3. `volume.rs`: adaptive box + rayon bake + peak normalize. Integration test: peak is non-zero, integral ≈ 1 for a few orbitals.
4. `render.rs`: upload one static 3D texture, hardcoded camera, ray-march fragment shader, single colormap. Get the `(3,2,1)` cloud on screen.
5. `camera.rs`: orbit + zoom + fit. Mouse + `F` key working.
6. egui integration: sliders for `(n,l,m)`, rebake on change. Confirm bounded-range UX.
7. Visual knobs: resolution dropdown, colormap dropdown (load all six LUTs), `k`/exposure sliders, auto-rotate.
8. HUD: scale bar, density readout, FPS.
9. Screenshot (`S` + button) → PNG via `image` crate (read back framebuffer).
10. Polish: presets dropdown, default-value tuning of `k`/exposure by eye, comment passes.

---

## 11. Reference-code "don't"s

Bugs and dead ends in the C++/Python reference. Do not port:

1. The Python `sample_points` rejection sampler uses a **running** `max_prob` denominator → biased. Irrelevant for us (no sampling) but flagged for completeness.
2. The C++ `sampleR`/`sampleTheta` cache CDF tables behind a `static bool built` that never resets → tables never rebuild on `(n,l,m)` change. Irrelevant for us (volume rebakes on every change automatically).
3. C++ color scaling uses magic constants (`LightingScaler = 700`, `pow(5, n)`). Replaced by per-orbital peak-normalize + `k` + exposure.
4. `calculateProbabilityFlow` is a stylized azimuthal current, not a real probability current. For real spherical harmonics the true probability current is identically zero. Camera auto-rotate replaces the "looks alive" intent without lying.

---

## 12. Deferred to v2

- 512³ as default → requires async bake + double-buffered 3D texture.
- Phase-as-hue mode for complex SH (would require a parallel complex pipeline).
- "Atlas" grid view of all valid `(n, l, m)` for `n ≤ 6`.
- "Compare ghost" overlay: lock current orbital as faint outline, scrub others over it.
- Volume rendering optimizations: jittered step start (anti-banding), early termination, empty-space skipping.

---

## 13. User working style (notes for whoever picks this up)

Accuracy-first, numbers-forward, direct. Wants claims verified rather than asserted (the physics above was verified for exactly this reason). Comfortable, experienced Rust developer — you can be terse and technical. Prefers PowerShell on Windows. Prefers caveman-mode communication: drop filler, articles, pleasantries; keep technical accuracy.
