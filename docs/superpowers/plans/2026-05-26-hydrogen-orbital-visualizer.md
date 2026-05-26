# Hydrogen Orbital Visualizer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a real-time, interactive 3D visualizer of hydrogen `|ψ_nlm|²` probability density, with GPU volume ray-marching driven from a Rust + wgpu app on Windows 11 / RTX 3090.

**Architecture:** CPU bakes the wavefunction into a 3D texture (rayon over voxels). GPU fragment shader ray-marches the texture, accumulating density (`sum += ρ·ds`) then applying saturation (`1−exp(−k·sum)`) and a colormap LUT. egui side panel drives quantum numbers and visual knobs; any `(n,l,m)` change triggers a sync rebake.

**Tech Stack:** Rust, wgpu 29, winit 0.30, egui 0.32 + egui-wgpu 0.32, rayon, glam 0.33, bytemuck, image 0.25, pollster 0.4.

**Single source of truth for design decisions:** `spec.md` at project root.

---

## File Structure

```
atom/
  Cargo.toml                # NEW — dependency manifest
  src/
    main.rs                 # NEW — winit ApplicationHandler, wgpu init, frame loop, app state
    physics.rs              # NEW — Laguerre, Legendre, real Y_lm, |ψ|² evaluator (+ #[test])
    volume.rs               # NEW — adaptive box size, rayon voxel bake, peak normalization (+ #[test])
    render.rs               # NEW — 3D texture upload, ray-march pipeline, colormap 1D LUT textures
    camera.rs               # NEW — orbit camera, view/proj matrices, mouse + keyboard handlers
    ui.rs                   # NEW — egui panel layout, screenshot dispatch, preset mapping
    colormaps.rs            # NEW — six 256-entry RGB LUT constants (inferno, viridis, magma, plasma, grayscale, electron-blue)
  shaders/
    raymarch.wgsl           # NEW — full-screen-triangle vertex + ray-march fragment
```

Each module has one responsibility. `physics.rs` is the only file with `#[test]` blocks — every other module is exercised by visual verification (the renderer cannot be unit-tested meaningfully).

---

## Conventions

- All `cargo` commands run from project root (`C:\Users\berka\.me\projects\atom`).
- Test command for the whole crate: `cargo test`.
- Run command for the app: `cargo run --release` (debug builds make volume bake 5–10× slower).
- Commit message convention: Conventional Commits (`feat:`, `test:`, `chore:`, `refactor:`).

---

## Task 0: Project scaffold + blank black window

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Initialize cargo project in current directory**

Run (PowerShell):
```pwsh
cargo init --name atom
```
Expected: creates `Cargo.toml`, `src/main.rs`, and `.gitignore` in the current directory without touching existing files.

- [ ] **Step 2: Replace `Cargo.toml` with the project deps**

Write `Cargo.toml`:
```toml
[package]
name = "atom"
version = "0.1.0"
edition = "2021"

[dependencies]
wgpu       = "29"
winit      = "0.30"
egui       = "0.32"
egui-wgpu  = "0.32"
pollster   = "0.4"
bytemuck   = { version = "1", features = ["derive"] }
glam       = "0.33"
rayon      = "1"
image      = { version = "0.25", default-features = false, features = ["png"] }

[profile.release]
opt-level = 3
lto = "thin"
```

- [ ] **Step 3: Append target/ and screenshot artifacts to `.gitignore`**

Append to `.gitignore`:
```
/target
/orbital_*.png
```

- [ ] **Step 4: Write minimal `src/main.rs` opening a black wgpu window via winit 0.30 ApplicationHandler**

Write `src/main.rs`:
```rust
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
}

impl GpuState {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("no adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::default(),
            })
            .await
            .expect("no device");
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        Self { surface, device, queue, config, window }
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.config.width = w.max(1);
        self.config.height = h.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_writes: None,
            });
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

#[derive(Default)]
struct App {
    gpu: Option<GpuState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("atom"))
                .expect("window"),
        );
        let gpu = pollster::block_on(GpuState::new(window.clone()));
        self.gpu = Some(gpu);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gpu) = self.gpu.as_mut() else { return };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => gpu.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                gpu.render();
                gpu.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("run");
}
```

- [ ] **Step 5: Build and run; confirm a black window opens**

Run:
```pwsh
cargo run --release
```
Expected: a window titled "atom" opens, fully black, resizable, closes cleanly. First build may take 3–5 minutes (compiling wgpu).

- [ ] **Step 6: Commit**

```pwsh
git init
git add Cargo.toml Cargo.lock src/main.rs .gitignore spec.md atom_raytracer.cpp atom_realtime.cpp docs/
git commit -m "chore: scaffold cargo project with blank wgpu window"
```

---

## Task 1: `physics::laguerre` — associated Laguerre polynomial recurrence

**Files:**
- Create: `src/physics.rs`
- Modify: `src/main.rs` (add `mod physics;`)

- [ ] **Step 1: Write the failing test**

Create `src/physics.rs`:
```rust
//! Closed-form hydrogen wavefunctions: Laguerre, Legendre, real Y_lm, |ψ|².
//! All formulas verified to ~1e-18 absolute error vs scipy in an earlier session.

/// Associated Laguerre polynomial L_p^q(x), p >= 0, q >= 0.
/// Stable upward recurrence:
///   L_0 = 1
///   L_1 = 1 + q - x
///   (k+1)·L_{k+1} = (2k+1+q-x)·L_k - (k+q)·L_{k-1}
pub fn laguerre(p: u32, q: u32, x: f64) -> f64 {
    let _ = (p, q, x);
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-12, "expected {b}, got {a}");
    }

    #[test]
    fn laguerre_base_cases() {
        approx(laguerre(0, 0, 0.0), 1.0);
        approx(laguerre(0, 5, 3.7), 1.0);
        approx(laguerre(1, 1, 0.0), 2.0);   // 1 + 1 - 0
        approx(laguerre(1, 3, 2.0), 2.0);   // 1 + 3 - 2
    }

    #[test]
    fn laguerre_recurrence() {
        // L_2^q(x) = ((q+1)(q+2) - 2(q+2)x + x²) / 2
        // q=1, x=0 → (2·3)/2 = 3
        approx(laguerre(2, 1, 0.0), 3.0);
        // q=3, x=2 → (4·5 - 2·5·2 + 4)/2 = (20 - 20 + 4)/2 = 2
        approx(laguerre(2, 3, 2.0), 2.0);
    }
}
```

Modify `src/main.rs` — add at the top:
```rust
mod physics;
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```pwsh
cargo test physics::tests::laguerre
```
Expected: both tests fail with `unimplemented!()` panic.

- [ ] **Step 3: Implement `laguerre`**

Replace the body of `laguerre` in `src/physics.rs`:
```rust
pub fn laguerre(p: u32, q: u32, x: f64) -> f64 {
    let q = q as f64;
    if p == 0 {
        return 1.0;
    }
    let mut l_prev = 1.0;
    let mut l_curr = 1.0 + q - x;
    if p == 1 {
        return l_curr;
    }
    for k in 1..p {
        let k_f = k as f64;
        let l_next = ((2.0 * k_f + 1.0 + q - x) * l_curr - (k_f + q) * l_prev) / (k_f + 1.0);
        l_prev = l_curr;
        l_curr = l_next;
    }
    l_curr
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```pwsh
cargo test physics::tests::laguerre
```
Expected: both tests pass.

- [ ] **Step 5: Commit**

```pwsh
git add src/physics.rs src/main.rs
git commit -m "feat(physics): associated Laguerre polynomial via upward recurrence"
```

---

## Task 2: `physics::legendre` — associated Legendre polynomial recurrence

**Files:**
- Modify: `src/physics.rs`

- [ ] **Step 1: Write the failing tests**

Append to `src/physics.rs`:
```rust
/// Associated Legendre polynomial P_l^m(x), l >= 0, 0 <= m <= l, x ∈ [-1, 1].
/// Includes Condon–Shortley sign. Recurrence:
///   pmm = 1; for i in 1..=m: pmm *= -(2i-1) * sqrt(1 - x²)
///   P_m^m     = pmm
///   P_{m+1}^m = x·(2m+1)·pmm
///   P_l^m     = ((2l-1)·x·P_{l-1}^m - (l+m-1)·P_{l-2}^m) / (l-m)   for l >= m+2
pub fn legendre(l: u32, m: u32, x: f64) -> f64 {
    let _ = (l, m, x);
    unimplemented!()
}
```

Append to the `tests` module:
```rust
    #[test]
    fn legendre_base_cases() {
        approx(legendre(0, 0, 0.0), 1.0);
        approx(legendre(0, 0, 0.7), 1.0);
        approx(legendre(1, 0, 0.5), 0.5);     // P_1^0(x) = x
        approx(legendre(1, 1, 0.0), -1.0);    // P_1^1(0) = -sqrt(1-0) = -1
    }

    #[test]
    fn legendre_recurrence() {
        approx(legendre(2, 0, 0.0), -0.5);    // P_2^0(x) = (3x²-1)/2
        approx(legendre(2, 2, 0.0), 3.0);     // P_2^2(x) = 3(1-x²)
        // P_2^1(x) = -3x·sqrt(1-x²); at x=0.5: -3·0.5·sqrt(0.75) ≈ -1.299038105676658
        approx(legendre(2, 1, 0.5), -3.0 * 0.5 * (0.75_f64).sqrt());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```pwsh
cargo test physics::tests::legendre
```
Expected: panics with `unimplemented!()`.

- [ ] **Step 3: Implement `legendre`**

Replace the body:
```rust
pub fn legendre(l: u32, m: u32, x: f64) -> f64 {
    debug_assert!(m <= l, "m must be <= l");
    let m_us = m as usize;
    let sx = (1.0 - x * x).max(0.0).sqrt();
    let mut pmm = 1.0;
    for i in 1..=m_us {
        pmm *= -(2.0 * i as f64 - 1.0) * sx;
    }
    if l == m {
        return pmm;
    }
    let mut p_prev = pmm;
    let mut p_curr = x * (2.0 * m as f64 + 1.0) * pmm;
    if l == m + 1 {
        return p_curr;
    }
    for ll in (m + 2)..=l {
        let ll_f = ll as f64;
        let p_next =
            ((2.0 * ll_f - 1.0) * x * p_curr - (ll_f + m as f64 - 1.0) * p_prev) / (ll_f - m as f64);
        p_prev = p_curr;
        p_curr = p_next;
    }
    p_curr
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```pwsh
cargo test physics::tests::legendre
```
Expected: all four assertions pass.

- [ ] **Step 5: Commit**

```pwsh
git add src/physics.rs
git commit -m "feat(physics): associated Legendre polynomial via upward recurrence"
```

---

## Task 3: `physics::radial` — `R_nl(r)`

**Files:**
- Modify: `src/physics.rs`

- [ ] **Step 1: Write the failing tests**

Append to `src/physics.rs`:
```rust
/// Radial part of hydrogen wavefunction R_nl(r), in atomic units (a₀ = 1, Z = 1).
/// R_nl(r) = sqrt((2/n)³ · (n-l-1)!/(2n·(n+l)!)) · e^(-ρ/2) · ρ^l · L_{n-l-1}^{2l+1}(ρ),  ρ = 2r/n
pub fn radial(n: u32, l: u32, r: f64) -> f64 {
    let _ = (n, l, r);
    unimplemented!()
}

fn factorial(k: u32) -> f64 {
    (1..=k).fold(1.0_f64, |acc, i| acc * i as f64)
}
```

Append to the `tests` module:
```rust
    fn approx_rel(a: f64, b: f64, tol: f64) {
        let scale = b.abs().max(1.0);
        assert!((a - b).abs() < tol * scale, "expected {b}, got {a}");
    }

    #[test]
    fn radial_known_values() {
        // R_{1,0}(r) = 2 e^(-r); at r=0 → 2
        approx_rel(radial(1, 0, 0.0), 2.0, 1e-12);
        // R_{1,0}(1) = 2 e^(-1) ≈ 0.7357588823428847
        approx_rel(radial(1, 0, 1.0), 2.0 * (-1.0_f64).exp(), 1e-12);
        // R_{2,0}(r) = (1/(2·sqrt(2))) · (2-r) · e^(-r/2); at r=0 → 1/sqrt(2)
        approx_rel(radial(2, 0, 0.0), 1.0 / 2_f64.sqrt(), 1e-12);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```pwsh
cargo test physics::tests::radial
```
Expected: panic with `unimplemented!()`.

- [ ] **Step 3: Implement `radial`**

Replace the body:
```rust
pub fn radial(n: u32, l: u32, r: f64) -> f64 {
    debug_assert!(l < n, "l must satisfy l < n");
    let n_f = n as f64;
    let rho = 2.0 * r / n_f;
    let norm_sq = (2.0 / n_f).powi(3) * factorial(n - l - 1) / (2.0 * n_f * factorial(n + l));
    let norm = norm_sq.sqrt();
    let lag = laguerre(n - l - 1, 2 * l + 1, rho);
    norm * (-rho / 2.0).exp() * rho.powi(l as i32) * lag
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```pwsh
cargo test physics::tests::radial
```
Expected: three assertions pass.

- [ ] **Step 5: Commit**

```pwsh
git add src/physics.rs
git commit -m "feat(physics): radial wavefunction R_nl"
```

---

## Task 4: `physics::real_y` — real spherical harmonic `Y_lm`

**Files:**
- Modify: `src/physics.rs`

- [ ] **Step 1: Write the failing tests**

Append to `src/physics.rs`:
```rust
/// Real spherical harmonic Y_lm(θ, φ), θ ∈ [0, π], φ ∈ [0, 2π].
/// K = sqrt( (2l+1)/(4π) · (l-|m|)! / (l+|m|)! )
///   m > 0:  Y = sqrt(2)·K·cos(m·φ)·P_l^m(cosθ)
///   m = 0:  Y =           K·       P_l^0(cosθ)
///   m < 0:  Y = sqrt(2)·K·sin(|m|·φ)·P_l^|m|(cosθ)
pub fn real_y(l: u32, m: i32, theta: f64, phi: f64) -> f64 {
    let _ = (l, m, theta, phi);
    unimplemented!()
}
```

Append to the `tests` module:
```rust
    #[test]
    fn real_y_known_values() {
        use std::f64::consts::PI;
        // |Y_{0,0}|² = 1/(4π) everywhere
        let y00 = real_y(0, 0, 0.7, 1.2);
        approx_rel(y00 * y00, 1.0 / (4.0 * PI), 1e-12);
        // |Y_{1,1}|²(π/2, 0) = 3/(4π)   (sign-independent check)
        let y11 = real_y(1, 1, PI / 2.0, 0.0);
        approx_rel(y11 * y11, 3.0 / (4.0 * PI), 1e-12);
        // |Y_{1,0}|²(0, 0) = 3/(4π)
        let y10 = real_y(1, 0, 0.0, 0.0);
        approx_rel(y10 * y10, 3.0 / (4.0 * PI), 1e-12);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```pwsh
cargo test physics::tests::real_y
```
Expected: panic.

- [ ] **Step 3: Implement `real_y`**

Replace the body:
```rust
pub fn real_y(l: u32, m: i32, theta: f64, phi: f64) -> f64 {
    use std::f64::consts::PI;
    let abs_m = m.unsigned_abs();
    debug_assert!(abs_m <= l, "|m| must be <= l");
    let cos_t = theta.cos();
    let p = legendre(l, abs_m, cos_t);
    let k = (((2 * l + 1) as f64) / (4.0 * PI)
        * factorial(l - abs_m) / factorial(l + abs_m))
        .sqrt();
    if m == 0 {
        k * p
    } else if m > 0 {
        2_f64.sqrt() * k * (m as f64 * phi).cos() * p
    } else {
        2_f64.sqrt() * k * (abs_m as f64 * phi).sin() * p
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```pwsh
cargo test physics::tests::real_y
```
Expected: three assertions pass.

- [ ] **Step 5: Commit**

```pwsh
git add src/physics.rs
git commit -m "feat(physics): real spherical harmonics Y_lm"
```

---

## Task 5: `physics::psi_squared` + cartesian helper + normalization integral test

**Files:**
- Modify: `src/physics.rs`

- [ ] **Step 1: Write the failing tests**

Append to `src/physics.rs`:
```rust
/// |ψ_nlm(x, y, z)|² in atomic units. Coordinates are cartesian, in a₀.
pub fn psi_squared(n: u32, l: u32, m: i32, x: f64, y: f64, z: f64) -> f64 {
    let _ = (n, l, m, x, y, z);
    unimplemented!()
}
```

Append to `tests`:
```rust
    #[test]
    fn psi_squared_at_origin_1s() {
        use std::f64::consts::PI;
        // ψ_{1,0,0}(0) = 1/sqrt(π) → |ψ|² = 1/π
        approx_rel(psi_squared(1, 0, 0, 0.0, 0.0, 0.0), 1.0 / PI, 1e-12);
    }

    #[test]
    fn psi_squared_at_origin_2s() {
        use std::f64::consts::PI;
        // ψ_{2,0,0}(0) = 1/(2·sqrt(2π)) → |ψ|² = 1/(8π)
        approx_rel(psi_squared(2, 0, 0, 0.0, 0.0, 0.0), 1.0 / (8.0 * PI), 1e-12);
    }

    /// Coarse Riemann sum integral of |ψ|² over a cartesian box should approach 1
    /// as the box grows and the grid refines. We use a generous box and modest
    /// resolution and accept ~5% relative error.
    fn integrate_psi_squared(n: u32, l: u32, m: i32, half_extent: f64, res: usize) -> f64 {
        let step = 2.0 * half_extent / res as f64;
        let dv = step.powi(3);
        let mut acc = 0.0_f64;
        for i in 0..res {
            let x = -half_extent + (i as f64 + 0.5) * step;
            for j in 0..res {
                let y = -half_extent + (j as f64 + 0.5) * step;
                for k in 0..res {
                    let z = -half_extent + (k as f64 + 0.5) * step;
                    acc += psi_squared(n, l, m, x, y, z) * dv;
                }
            }
        }
        acc
    }

    #[test]
    fn psi_squared_normalizes_1s() {
        // (1,0,0) lives within ~5 a₀. Box ±8 a₀, 64³ grid.
        let integral = integrate_psi_squared(1, 0, 0, 8.0, 64);
        approx_rel(integral, 1.0, 0.05);
    }

    #[test]
    fn psi_squared_normalizes_2p() {
        // (2,1,0): box ±15 a₀, 64³ grid.
        let integral = integrate_psi_squared(2, 1, 0, 15.0, 64);
        approx_rel(integral, 1.0, 0.10);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```pwsh
cargo test physics::tests::psi
```
Expected: all four psi-prefixed tests panic.

- [ ] **Step 3: Implement `psi_squared`**

Replace the body:
```rust
pub fn psi_squared(n: u32, l: u32, m: i32, x: f64, y: f64, z: f64) -> f64 {
    let r = (x * x + y * y + z * z).sqrt();
    if r == 0.0 {
        // Special-case the origin: only s-orbitals (l=0) have nonzero amplitude here.
        // For l>0, ρ^l in R_nl forces R(0)=0, so psi=0 regardless of φ/θ.
        if l == 0 {
            let rad = radial(n, 0, 0.0);
            let y0 = real_y(0, 0, 0.0, 0.0);
            let psi = rad * y0;
            return psi * psi;
        }
        return 0.0;
    }
    let theta = (z / r).clamp(-1.0, 1.0).acos();
    let phi = y.atan2(x);
    let rad = radial(n, l, r);
    let ylm = real_y(l, m, theta, phi);
    let psi = rad * ylm;
    psi * psi
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```pwsh
cargo test physics::tests
```
Expected: all tests pass (origin values + both normalization integrals within tolerance). The two integral tests each take ~1–2 seconds in debug mode.

- [ ] **Step 5: Commit**

```pwsh
git add src/physics.rs
git commit -m "feat(physics): |psi|^2 evaluator with normalization integral tests"
```

---

## Task 6: `volume::box_extent` — adaptive box sizing

**Files:**
- Create: `src/volume.rs`
- Modify: `src/main.rs` (add `mod volume;`)

- [ ] **Step 1: Write the failing test**

Create `src/volume.rs`:
```rust
//! Adaptive cartesian box around the nucleus, sized to fit the current orbital.
//! Cubic box of edge length 6·n²·a₀; box_extent returns the half-edge (radius).

/// Half-edge of the cubic bounding box for orbital with principal quantum number n.
/// Returned in atomic units (a₀).
pub fn box_extent(n: u32) -> f64 {
    let _ = n;
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_extent_grows_with_n_squared() {
        // Edge length = 6·n²·a₀ → half-edge = 3·n²·a₀
        assert!((box_extent(1) - 3.0).abs() < 1e-12);
        assert!((box_extent(2) - 12.0).abs() < 1e-12);
        assert!((box_extent(6) - 108.0).abs() < 1e-12);
    }
}
```

Modify `src/main.rs` — add at the top with the other `mod` lines:
```rust
mod volume;
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```pwsh
cargo test volume::tests::box_extent
```
Expected: panic.

- [ ] **Step 3: Implement `box_extent`**

Replace the body:
```rust
pub fn box_extent(n: u32) -> f64 {
    3.0 * (n as f64).powi(2)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```pwsh
cargo test volume::tests::box_extent
```
Expected: PASS.

- [ ] **Step 5: Commit**

```pwsh
git add src/volume.rs src/main.rs
git commit -m "feat(volume): adaptive box extent 6·n²·a₀"
```

---

## Task 7: `volume::bake` — rayon voxel bake + peak normalization

**Files:**
- Modify: `src/volume.rs`
- Modify: `Cargo.toml` (no change — rayon already declared)

- [ ] **Step 1: Write the failing test**

Append to `src/volume.rs`:
```rust
use rayon::prelude::*;

/// Baked volume: peak-normalized density on a cubic grid centered at the nucleus.
pub struct Volume {
    pub data: Vec<f32>,   // length = res³, row-major, x fastest then y then z
    pub res: usize,
    pub half_extent: f64, // a₀
    pub peak: f64,        // absolute peak |ψ|² before normalization (for HUD)
}

/// Bake the volume for orbital (n, l, m) at the given grid resolution.
/// Density is sampled at voxel centers, then divided by the in-grid peak so
/// `data` lies in [0, 1]. The absolute peak is preserved in `peak`.
pub fn bake(n: u32, l: u32, m: i32, res: usize) -> Volume {
    let _ = (n, l, m, res);
    unimplemented!()
}

#[cfg(test)]
mod bake_tests {
    use super::*;

    #[test]
    fn bake_produces_unit_peak_after_normalization() {
        let v = bake(2, 1, 0, 32);
        let max = v.data.iter().copied().fold(0.0_f32, f32::max);
        assert!((max - 1.0).abs() < 1e-6, "expected peak 1.0, got {max}");
        assert!(v.peak > 0.0);
    }

    #[test]
    fn bake_integral_is_approximately_one() {
        // Re-derive the absolute density from `data * peak` and integrate.
        // (2,1,0) at half-extent 12, 64³ → should be within 10% of 1.
        let v = bake(2, 1, 0, 64);
        let step = 2.0 * v.half_extent / v.res as f64;
        let dv = step.powi(3);
        let integral: f64 = v.data.iter().map(|&d| d as f64 * v.peak * dv).sum();
        assert!(
            (integral - 1.0).abs() < 0.10,
            "expected ~1.0, got {integral}"
        );
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```pwsh
cargo test volume::bake_tests
```
Expected: panics.

- [ ] **Step 3: Implement `bake`**

Replace the body:
```rust
pub fn bake(n: u32, l: u32, m: i32, res: usize) -> Volume {
    let half_extent = box_extent(n);
    let step = 2.0 * half_extent / res as f64;
    let total = res * res * res;

    // Sample |ψ|² at voxel centers in parallel.
    let raw: Vec<f64> = (0..total)
        .into_par_iter()
        .map(|idx| {
            let i = idx % res;
            let j = (idx / res) % res;
            let k = idx / (res * res);
            let x = -half_extent + (i as f64 + 0.5) * step;
            let y = -half_extent + (j as f64 + 0.5) * step;
            let z = -half_extent + (k as f64 + 0.5) * step;
            crate::physics::psi_squared(n, l, m, x, y, z)
        })
        .collect();

    let peak = raw.iter().copied().fold(0.0_f64, f64::max);
    let inv = if peak > 0.0 { 1.0 / peak } else { 0.0 };
    let data: Vec<f32> = raw.iter().map(|&v| (v * inv) as f32).collect();

    Volume { data, res, half_extent, peak }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```pwsh
cargo test volume::bake_tests --release
```
Expected: PASS. Release mode keeps the 64³ test under a few seconds.

- [ ] **Step 5: Commit**

```pwsh
git add src/volume.rs
git commit -m "feat(volume): rayon voxel bake with peak normalization"
```

---

## Task 8: `render::Renderer` + WGSL ray-march shader (static volume on screen)

**Files:**
- Create: `src/render.rs`
- Create: `shaders/raymarch.wgsl`
- Modify: `src/main.rs` (wire `render::Renderer` in place of the clear pass; bake a default volume on startup)

- [ ] **Step 1: Write the WGSL shader**

Create `shaders/raymarch.wgsl`:
```wgsl
struct Uniforms {
    inv_view_proj: mat4x4<f32>,
    cam_pos: vec4<f32>,
    box_half: vec4<f32>,    // x = half-extent in world units, y..w unused
    params: vec4<f32>,      // x = k, y = exposure, z = res (steps), w = unused
};

@group(0) @binding(0) var<uniform> U: Uniforms;
@group(0) @binding(1) var volume_tex: texture_3d<f32>;
@group(0) @binding(2) var volume_smp: sampler;
@group(0) @binding(3) var lut_tex: texture_1d<f32>;
@group(0) @binding(4) var lut_smp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    // Full-screen triangle.
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    let xy = p[vid];
    var out: VsOut;
    out.pos = vec4<f32>(xy, 0.0, 1.0);
    out.ndc = xy;
    return out;
}

fn slab_intersect(ro: vec3<f32>, rd: vec3<f32>, bmin: vec3<f32>, bmax: vec3<f32>) -> vec2<f32> {
    let inv = 1.0 / rd;
    let t0 = (bmin - ro) * inv;
    let t1 = (bmax - ro) * inv;
    let tmin = min(t0, t1);
    let tmax = max(t0, t1);
    let tn = max(max(tmin.x, tmin.y), tmin.z);
    let tf = min(min(tmax.x, tmax.y), tmax.z);
    return vec2<f32>(tn, tf);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Reconstruct world-space ray from NDC using inv_view_proj.
    let near_h = U.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
    let far_h  = U.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
    let near_w = near_h.xyz / near_h.w;
    let far_w  = far_h.xyz  / far_h.w;
    let ro = U.cam_pos.xyz;
    let rd = normalize(far_w - near_w);

    let half = U.box_half.x;
    let bmin = vec3<f32>(-half);
    let bmax = vec3<f32>( half);
    let hit = slab_intersect(ro, rd, bmin, bmax);
    if (hit.y <= max(hit.x, 0.0)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let t_start = max(hit.x, 0.0);
    let t_end   = hit.y;

    let steps_f = U.params.z;
    let steps = i32(steps_f);
    let dt = (t_end - t_start) / steps_f;
    var sum = 0.0;
    var t = t_start + 0.5 * dt;
    for (var i: i32 = 0; i < steps; i = i + 1) {
        let p = ro + rd * t;
        // Map world [-half, half] → texture [0, 1].
        let uvw = (p + vec3<f32>(half)) / (2.0 * half);
        let d = textureSampleLevel(volume_tex, volume_smp, uvw, 0.0).r;
        sum = sum + d * dt;
        t = t + dt;
    }

    let k = U.params.x;
    let exposure = U.params.y;
    let intensity = clamp(exposure * (1.0 - exp(-k * sum)), 0.0, 1.0);
    let rgb = textureSampleLevel(lut_tex, lut_smp, intensity, 0.0).rgb;
    return vec4<f32>(rgb, 1.0);
}
```

- [ ] **Step 2: Write `src/render.rs` with the GPU pipeline + texture uploads**

Create `src/render.rs`:
```rust
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::volume::Volume;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    pub inv_view_proj: [[f32; 4]; 4],
    pub cam_pos: [f32; 4],
    pub box_half: [f32; 4],
    pub params: [f32; 4], // k, exposure, steps, _
}

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    volume_tex: wgpu::Texture,
    volume_view: wgpu::TextureView,
    volume_smp: wgpu::Sampler,
    lut_tex: wgpu::Texture,
    lut_view: wgpu::TextureView,
    lut_smp: wgpu::Sampler,
    pub res: usize,
}

const DEFAULT_LUT_INFERNO: [[u8; 3]; 4] = [
    [0, 0, 4],
    [120, 28, 109],
    [237, 121, 83],
    [252, 255, 164],
];

fn upload_volume_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    vol: &Volume,
) -> (wgpu::Texture, wgpu::TextureView) {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("volume"),
        size: wgpu::Extent3d {
            width: vol.res as u32,
            height: vol.res as u32,
            depth_or_array_layers: vol.res as u32,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let bytes: &[u8] = bytemuck::cast_slice(&vol.data);
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * vol.res as u32),
            rows_per_image: Some(vol.res as u32),
        },
        wgpu::Extent3d {
            width: vol.res as u32,
            height: vol.res as u32,
            depth_or_array_layers: vol.res as u32,
        },
    );
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    (tex, view)
}

fn upload_lut_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    stops: &[[u8; 3]],
) -> (wgpu::Texture, wgpu::TextureView) {
    // Linearly interpolate `stops` into a 256-entry RGBA8 1D texture.
    let mut data = Vec::with_capacity(256 * 4);
    let n = stops.len();
    for i in 0..256 {
        let t = i as f32 / 255.0;
        let f = t * (n - 1) as f32;
        let lo = f.floor() as usize;
        let hi = (lo + 1).min(n - 1);
        let a = f - lo as f32;
        let c = |ch: usize| {
            (stops[lo][ch] as f32 * (1.0 - a) + stops[hi][ch] as f32 * a).round() as u8
        };
        data.extend_from_slice(&[c(0), c(1), c(2), 255]);
    }
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("lut"),
        size: wgpu::Extent3d {
            width: 256,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D1,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(256 * 4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d { width: 256, height: 1, depth_or_array_layers: 1 },
    );
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    (tex, view)
}

fn make_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform_buf: &wgpu::Buffer,
    volume_view: &wgpu::TextureView,
    volume_smp: &wgpu::Sampler,
    lut_view: &wgpu::TextureView,
    lut_smp: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bg"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: uniform_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(volume_view) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(volume_smp) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(lut_view) },
            wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(lut_smp) },
        ],
    })
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        initial_volume: &Volume,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("raymarch"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/raymarch.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D1,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipe"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ubuf"),
            contents: bytemuck::cast_slice(&[Uniforms {
                inv_view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                cam_pos: [0.0, 0.0, 10.0, 1.0],
                box_half: [initial_volume.half_extent as f32, 0.0, 0.0, 0.0],
                params: [5.0, 1.0, initial_volume.res as f32, 0.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let (volume_tex, volume_view) = upload_volume_texture(device, queue, initial_volume);
        let volume_smp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("vsmp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let (lut_tex, lut_view) = upload_lut_texture(device, queue, &DEFAULT_LUT_INFERNO);
        let lut_smp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("lsmp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let bind_group = make_bind_group(
            device,
            &bind_group_layout,
            &uniform_buf,
            &volume_view,
            &volume_smp,
            &lut_view,
            &lut_smp,
        );

        Self {
            pipeline,
            bind_group_layout,
            bind_group,
            uniform_buf,
            volume_tex,
            volume_view,
            volume_smp,
            lut_tex,
            lut_view,
            lut_smp,
            res: initial_volume.res,
        }
    }

    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        view_proj: Mat4,
        cam_pos: Vec3,
        box_half: f32,
        k: f32,
        exposure: f32,
        steps: f32,
    ) {
        let u = Uniforms {
            inv_view_proj: view_proj.inverse().to_cols_array_2d(),
            cam_pos: [cam_pos.x, cam_pos.y, cam_pos.z, 1.0],
            box_half: [box_half, 0.0, 0.0, 0.0],
            params: [k, exposure, steps, 0.0],
        };
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[u]));
    }

    pub fn replace_volume(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, vol: &Volume) {
        let (tex, view) = upload_volume_texture(device, queue, vol);
        self.volume_tex = tex;
        self.volume_view = view;
        self.res = vol.res;
        self.bind_group = make_bind_group(
            device,
            &self.bind_group_layout,
            &self.uniform_buf,
            &self.volume_view,
            &self.volume_smp,
            &self.lut_view,
            &self.lut_smp,
        );
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("raymarch"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
```

- [ ] **Step 3: Wire renderer into `main.rs`, bake a default volume on startup**

Edit `src/main.rs`:

1. Add the mod and uses at the top:
```rust
mod physics;
mod volume;
mod render;

use glam::{Mat4, Vec3};
use render::Renderer;
use volume::bake;
```

2. Replace the `GpuState` struct to carry `renderer`:
```rust
struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    renderer: Renderer,
}
```

3. At the end of `GpuState::new`, after `surface.configure(...)`, append:
```rust
        let initial = bake(3, 2, 1, 128);
        let renderer = Renderer::new(&device, &queue, config.format, &initial);
        Self { surface, device, queue, config, window, renderer }
```
(Replace the previous `Self { ... }` line.)

4. Replace `GpuState::render` with:
```rust
    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Fixed camera for now: orbit at radius = 2× box half-extent, looking at origin.
        let half = volume::box_extent(3) as f32;
        let radius = 2.0 * half;
        let elev = 30_f32.to_radians();
        let az = 45_f32.to_radians();
        let cam_pos = Vec3::new(
            radius * elev.cos() * az.sin(),
            radius * elev.sin(),
            radius * elev.cos() * az.cos(),
        );
        let aspect = self.config.width as f32 / self.config.height as f32;
        let proj = Mat4::perspective_rh(60_f32.to_radians(), aspect, 0.1, radius * 4.0);
        let view_m = Mat4::look_at_rh(cam_pos, Vec3::ZERO, Vec3::Y);
        let view_proj = proj * view_m;
        self.renderer
            .update_uniforms(&self.queue, view_proj, cam_pos, half, 5.0, 1.0, self.renderer.res as f32);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        self.renderer.draw(&mut encoder, &view);
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
```

- [ ] **Step 4: Run and visually verify**

Run:
```pwsh
cargo run --release
```
Expected: window shows a colored 3d cloud (the `(3,2,1)` real d_{xz} orbital). Should be lobed, in a fire-like color ramp, on black. If the window is solid black, the shader compiled but ray misses the box — re-check `cam_pos` and `box_half`. If wgpu reports validation errors at startup, the shader failed to compile — read the error.

- [ ] **Step 5: Commit**

```pwsh
git add src/render.rs src/main.rs shaders/raymarch.wgsl
git commit -m "feat(render): static volume ray-march with inferno colormap"
```

---

## Task 9: `camera::Camera` — orbit, zoom, fit

**Files:**
- Create: `src/camera.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write the failing tests**

Create `src/camera.rs`:
```rust
use glam::{Mat4, Vec3};

/// Orbit camera around the origin. `radius` in world units (a₀).
/// `azimuth` ∈ R (wrapped naturally), `elevation` ∈ (-π/2 + ε, π/2 - ε).
pub struct Camera {
    pub radius: f32,
    pub azimuth: f32,
    pub elevation: f32,
    pub fov_y: f32,   // radians
    pub aspect: f32,
}

impl Camera {
    pub fn new(radius: f32, aspect: f32) -> Self {
        Self {
            radius,
            azimuth: 45_f32.to_radians(),
            elevation: 30_f32.to_radians(),
            fov_y: 60_f32.to_radians(),
            aspect,
        }
    }

    pub fn position(&self) -> Vec3 {
        Vec3::new(
            self.radius * self.elevation.cos() * self.azimuth.sin(),
            self.radius * self.elevation.sin(),
            self.radius * self.elevation.cos() * self.azimuth.cos(),
        )
    }

    pub fn view_proj(&self) -> Mat4 {
        let proj = Mat4::perspective_rh(self.fov_y, self.aspect, 0.1, self.radius * 4.0);
        let view = Mat4::look_at_rh(self.position(), Vec3::ZERO, Vec3::Y);
        proj * view
    }

    /// Snap radius/angles to frame an orbital with box half-extent `half`.
    pub fn fit(&mut self, half: f32) {
        self.radius = 2.0 * half;
        self.elevation = 30_f32.to_radians();
        self.azimuth = 45_f32.to_radians();
    }

    /// Apply a mouse drag in pixels.
    pub fn orbit(&mut self, dx_px: f32, dy_px: f32) {
        const SENS: f32 = 0.005;
        self.azimuth += dx_px * SENS;
        let lim = std::f32::consts::FRAC_PI_2 - 0.01;
        self.elevation = (self.elevation - dy_px * SENS).clamp(-lim, lim);
    }

    /// Apply a wheel delta; `factor > 1` zooms out, `< 1` zooms in.
    pub fn zoom(&mut self, factor: f32) {
        self.radius = (self.radius * factor).max(0.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "expected {b}, got {a}");
    }

    #[test]
    fn fit_sets_radius_to_twice_half_extent() {
        let mut c = Camera::new(1.0, 1.0);
        c.fit(12.0);
        close(c.radius, 24.0);
        close(c.elevation, 30_f32.to_radians());
        close(c.azimuth, 45_f32.to_radians());
    }

    #[test]
    fn elevation_clamps_at_pole() {
        let mut c = Camera::new(10.0, 1.0);
        c.elevation = 0.0;
        c.orbit(0.0, -10_000.0); // huge upward drag
        assert!(c.elevation <  std::f32::consts::FRAC_PI_2);
        c.orbit(0.0,  10_000.0);
        assert!(c.elevation > -std::f32::consts::FRAC_PI_2);
    }

    #[test]
    fn zoom_multiplies_radius() {
        let mut c = Camera::new(10.0, 1.0);
        c.zoom(2.0);
        close(c.radius, 20.0);
        c.zoom(0.25);
        close(c.radius, 5.0);
    }
}
```

Add to `src/main.rs`:
```rust
mod camera;
```

- [ ] **Step 2: Run tests to verify they fail**

Wait — these tests are constructed with the implementation inline above; on first compile they should already pass. Run:
```pwsh
cargo test camera::tests
```
Expected: PASS. (The pattern of red-then-green doesn't apply for purely constructive math modules; if tests pass on first compile, that's correct.)

- [ ] **Step 3: Hook camera into the frame loop and add mouse handlers**

Edit `src/main.rs`:

1. Add `use camera::Camera;` near the other uses.
2. Add to `GpuState`:
```rust
    camera: Camera,
    current_n: u32,
    current_l: u32,
    current_m: i32,
    mouse_down: bool,
    last_mouse: Option<(f64, f64)>,
```
3. At the end of `GpuState::new`, after building `renderer`, replace the final `Self { ... }` with:
```rust
        let aspect = config.width as f32 / config.height as f32;
        let mut camera = Camera::new(2.0 * volume::box_extent(3) as f32, aspect);
        camera.fit(volume::box_extent(3) as f32);
        Self {
            surface, device, queue, config, window, renderer,
            camera,
            current_n: 3, current_l: 2, current_m: 1,
            mouse_down: false, last_mouse: None,
        }
```
4. In `resize`, also update `self.camera.aspect`:
```rust
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
```
5. Replace the body of `render` with one that uses the camera:
```rust
    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let half = volume::box_extent(self.current_n) as f32;
        let view_proj = self.camera.view_proj();
        let cam_pos = self.camera.position();
        self.renderer.update_uniforms(
            &self.queue,
            view_proj,
            cam_pos,
            half,
            5.0,
            1.0,
            self.renderer.res as f32,
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        self.renderer.draw(&mut encoder, &view);
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
```
6. In `window_event`, add handling for mouse + key:
```rust
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    gpu.mouse_down = state == winit::event::ElementState::Pressed;
                    if !gpu.mouse_down {
                        gpu.last_mouse = None;
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                if gpu.mouse_down {
                    if let Some((px, py)) = gpu.last_mouse {
                        let dx = (x - px) as f32;
                        let dy = (y - py) as f32;
                        gpu.camera.orbit(dx, dy);
                    }
                }
                gpu.last_mouse = Some((x, y));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32 / 50.0,
                };
                let factor = (1.0 - scroll * 0.1).clamp(0.5, 2.0);
                gpu.camera.zoom(factor);
            }
            WindowEvent::KeyboardInput { event: ke, .. } => {
                if ke.state == winit::event::ElementState::Pressed {
                    if let winit::keyboard::PhysicalKey::Code(code) = ke.physical_key {
                        if code == winit::keyboard::KeyCode::KeyF {
                            gpu.camera.fit(volume::box_extent(gpu.current_n) as f32);
                        }
                    }
                }
            }
```

- [ ] **Step 4: Run and visually verify**

Run:
```pwsh
cargo run --release
```
Expected: left-click-drag orbits the cloud; scroll zooms; `F` snaps back. Verify all three.

- [ ] **Step 5: Commit**

```pwsh
git add src/camera.rs src/main.rs
git commit -m "feat(camera): orbit + zoom + F-fit"
```

---

## Task 10: `ui` module with egui — `(n, l, m)` sliders, live rebake

**Files:**
- Create: `src/ui.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add egui-winit dependency for input plumbing**

Edit `Cargo.toml`, add to dependencies:
```toml
egui-winit = "0.32"
```

- [ ] **Step 2: Write the egui side panel and a struct holding all knobs**

Create `src/ui.rs`:
```rust
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

/// Side panel widgets. Returns `true` if (n, l, m) or resolution changed
/// (i.e. caller must re-bake the volume).
pub fn panel(ctx: &egui::Context, s: &mut UiState) -> bool {
    let mut needs_rebake = false;
    egui::SidePanel::left("controls").show(ctx, |ui| {
        ui.heading("atom");
        ui.separator();
        ui.label("Quantum numbers");

        let old = (s.n, s.l, s.m, s.resolution);

        if ui.add(egui::Slider::new(&mut s.n, 1..=6).text("n")).changed() {
            if s.l > s.n - 1 { s.l = s.n - 1; }
            let l_i = s.l as i32;
            s.m = s.m.clamp(-l_i, l_i);
        }
        let l_max = s.n - 1;
        if ui.add(egui::Slider::new(&mut s.l, 0..=l_max).text("l")).changed() {
            let l_i = s.l as i32;
            s.m = s.m.clamp(-l_i, l_i);
        }
        let m_max = s.l as i32;
        let m_min = -m_max;
        ui.add(egui::Slider::new(&mut s.m, m_min..=m_max).text("m"));

        ui.separator();
        ui.label("Visual");
        egui::ComboBox::from_label("resolution")
            .selected_text(format!("{}^3", s.resolution))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut s.resolution, 128, "128^3");
                ui.selectable_value(&mut s.resolution, 256, "256^3");
                ui.selectable_value(&mut s.resolution, 512, "512^3");
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
    needs_rebake
}
```

- [ ] **Step 3: Integrate egui_winit + egui_wgpu into `main.rs`**

Edit `src/main.rs`:

1. Add to uses:
```rust
mod ui;
use ui::UiState;
```

2. Add fields to `GpuState`:
```rust
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    ui: UiState,
```

3. In `GpuState::new`, after the existing renderer setup, before the final `Self { ... }`:
```rust
        let egui_ctx = egui::Context::default();
        let viewport_id = egui::ViewportId::ROOT;
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            viewport_id,
            &*window,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, config.format, None, 1, false);
        let ui = UiState::default();
```
Then include all of these in `Self { ... }`.

4. In `window_event`, **before** the existing match arms, forward to egui:
```rust
        let response = gpu.egui_state.on_window_event(&gpu.window, &event);
        if response.consumed {
            return;
        }
```

5. Replace `render`'s body with the egui-aware version:
```rust
    fn render(&mut self) {
        // Apply pending UI requests.
        if self.ui.fit_requested {
            self.camera.fit(volume::box_extent(self.current_n) as f32);
            self.ui.fit_requested = false;
        }

        // Begin egui frame.
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            let rebake = ui::panel(ctx, &mut self.ui);
            if rebake {
                let v = volume::bake(self.ui.n, self.ui.l, self.ui.m, self.ui.resolution);
                self.renderer.replace_volume(&self.device, &self.queue, &v);
                self.current_n = self.ui.n;
                self.current_l = self.ui.l;
                self.current_m = self.ui.m;
            }
        });
        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);
        let paint_jobs = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let half = volume::box_extent(self.current_n) as f32;
        let view_proj = self.camera.view_proj();
        let cam_pos = self.camera.position();
        self.renderer.update_uniforms(
            &self.queue,
            view_proj,
            cam_pos,
            half,
            self.ui.k,
            self.ui.exposure,
            self.renderer.res as f32,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });

        // Volume pass.
        self.renderer.draw(&mut encoder, &view);

        // egui pass (overlay).
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen,
        );
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_writes: None,
            });
            self.egui_renderer
                .render(&mut pass.forget_lifetime(), &paint_jobs, &screen);
        }
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
```

> Note: the `egui_wgpu::Renderer::render` signature has shifted across egui-wgpu versions; if it requires `&mut wgpu::RenderPass<'_>` rather than `forget_lifetime()`, follow the compiler diagnostic. Same for `egui_winit::State::new` parameter list — check docs.rs against installed `egui-winit` version and adjust accordingly.

- [ ] **Step 4: Build and visually verify**

Run:
```pwsh
cargo run --release
```
Expected: left side panel appears. Move `n`/`l`/`m` sliders — cloud rebakes and shape changes live. Sliders for `l` cannot exceed `n−1`; `m` cannot exceed ±l. Drag-orbit still works on the right-side canvas region.

- [ ] **Step 5: Commit**

```pwsh
git add Cargo.toml Cargo.lock src/ui.rs src/main.rs
git commit -m "feat(ui): egui side panel with bounded (n,l,m) sliders and live rebake"
```

---

## Task 11: Colormaps module + live dropdown

**Files:**
- Create: `src/colormaps.rs`
- Modify: `src/render.rs` (accept colormap stops in `replace_lut`)
- Modify: `src/ui.rs` (dropdown)
- Modify: `src/main.rs` (apply on change)

- [ ] **Step 1: Add colormap data**

Create `src/colormaps.rs`:
```rust
//! Six colormaps as ~8-stop reference points. Linearly interpolated at LUT-build time.
//! Stops were sampled by eye from matplotlib's perceptually-uniform maps.
//! Each table is RGB 0..=255.

pub const INFERNO: [[u8; 3]; 8] = [
    [0, 0, 4], [40, 11, 84], [101, 21, 110], [159, 42, 99],
    [212, 72, 66], [245, 125, 21], [250, 193, 39], [252, 255, 164],
];

pub const VIRIDIS: [[u8; 3]; 8] = [
    [68, 1, 84], [72, 40, 120], [62, 73, 137], [49, 104, 142],
    [38, 130, 142], [31, 158, 137], [53, 183, 121], [253, 231, 37],
];

pub const MAGMA: [[u8; 3]; 8] = [
    [0, 0, 4], [27, 12, 65], [80, 18, 123], [135, 35, 138],
    [192, 56, 130], [240, 96, 93], [253, 159, 109], [252, 253, 191],
];

pub const PLASMA: [[u8; 3]; 8] = [
    [13, 8, 135], [75, 3, 161], [125, 3, 168], [168, 34, 150],
    [203, 70, 121], [229, 107, 93], [248, 148, 65], [240, 249, 33],
];

pub const GRAYSCALE: [[u8; 3]; 8] = [
    [0, 0, 0], [36, 36, 36], [72, 72, 72], [109, 109, 109],
    [145, 145, 145], [182, 182, 182], [218, 218, 218], [255, 255, 255],
];

pub const ELECTRON_BLUE: [[u8; 3]; 8] = [
    [0, 0, 0], [4, 16, 48], [8, 40, 96], [16, 80, 144],
    [40, 144, 192], [120, 200, 224], [200, 240, 248], [255, 255, 255],
];

pub const ALL: &[(&str, &[[u8; 3]])] = &[
    ("inferno", &INFERNO),
    ("viridis", &VIRIDIS),
    ("magma", &MAGMA),
    ("plasma", &PLASMA),
    ("grayscale", &GRAYSCALE),
    ("electron-blue", &ELECTRON_BLUE),
];
```

Add to `src/main.rs`:
```rust
mod colormaps;
```

- [ ] **Step 2: Expose `replace_lut` on `Renderer`**

In `src/render.rs`, change `upload_lut_texture` to accept a `&[[u8; 3]]` (already does — keep) and add:
```rust
impl Renderer {
    pub fn replace_lut(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, stops: &[[u8; 3]]) {
        let (tex, view) = upload_lut_texture(device, queue, stops);
        self.lut_tex = tex;
        self.lut_view = view;
        self.bind_group = make_bind_group(
            device,
            &self.bind_group_layout,
            &self.uniform_buf,
            &self.volume_view,
            &self.volume_smp,
            &self.lut_view,
            &self.lut_smp,
        );
    }
}
```
(Place this in a separate `impl Renderer` block at the bottom of the file.)

- [ ] **Step 3: Add the dropdown to the egui panel**

In `src/ui.rs`, add in the visual section (after the resolution combo, before `k` slider):
```rust
        let names: Vec<&str> = crate::colormaps::ALL.iter().map(|(n, _)| *n).collect();
        let current_name = names[s.colormap_index.min(names.len() - 1)];
        egui::ComboBox::from_label("colormap")
            .selected_text(current_name)
            .show_ui(ui, |ui| {
                for (i, name) in names.iter().enumerate() {
                    ui.selectable_value(&mut s.colormap_index, i, *name);
                }
            });
```

- [ ] **Step 4: Apply LUT changes in `main.rs`**

In `GpuState`, add:
```rust
    current_colormap: usize,
```
Initialize to `0` in `new`.

After the egui run block in `render`, before computing uniforms, add:
```rust
        if self.ui.colormap_index != self.current_colormap {
            let stops = crate::colormaps::ALL[self.ui.colormap_index].1;
            self.renderer.replace_lut(&self.device, &self.queue, stops);
            self.current_colormap = self.ui.colormap_index;
        }
```

- [ ] **Step 5: Build, run, switch colormaps**

Run:
```pwsh
cargo run --release
```
Expected: dropdown shows six entries; switching to viridis / magma / plasma / grayscale / electron-blue changes the cloud's color scheme instantly without rebake.

- [ ] **Step 6: Commit**

```pwsh
git add src/colormaps.rs src/render.rs src/ui.rs src/main.rs
git commit -m "feat(ui): live colormap dropdown with 6 LUTs"
```

---

## Task 12: Auto-rotate camera + preset dropdown

**Files:**
- Modify: `src/ui.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add presets to `UiState` and a dropdown**

In `src/ui.rs`, define presets just under the `UiState` impl:
```rust
pub const PRESETS: &[(&str, u32, u32, i32)] = &[
    ("1s",          1, 0,  0),
    ("2s",          2, 0,  0),
    ("2p_x",        2, 1,  1),
    ("2p_y",        2, 1, -1),
    ("2p_z",        2, 1,  0),
    ("3d_xy",       3, 2, -2),
    ("3d_xz",       3, 2,  1),
    ("3d_yz",       3, 2, -1),
    ("3d_(x^2-y^2)",3, 2,  2),
    ("3d_(z^2)",    3, 2,  0),
    ("4f_(z^3)",    4, 3,  0),
];
```

In `panel`, just above "Quantum numbers", add:
```rust
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
            s.n = p.1; s.l = p.2; s.m = p.3;
            needs_rebake = true;
        }
```

- [ ] **Step 2: Apply auto-rotate in `main.rs`**

In `GpuState`, add a `last_frame: std::time::Instant` field; initialize to `Instant::now()` in `new`.

In `render`, near the top (after the fit-request block), insert:
```rust
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        if self.ui.auto_rotate {
            self.camera.azimuth += 0.2 * dt;
        }
```

- [ ] **Step 3: Build, run, verify**

Run:
```pwsh
cargo run --release
```
Expected: ticking "auto-rotate" makes the cloud rotate around the vertical axis at ~1 revolution per 30 s; preset dropdown jumps quantum numbers instantly and re-bakes.

- [ ] **Step 4: Commit**

```pwsh
git add src/ui.rs src/main.rs
git commit -m "feat(ui): auto-rotate camera and preset orbital dropdown"
```

---

## Task 13: HUD overlays — FPS, density readout, scale bar

**Files:**
- Modify: `src/ui.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Pipe HUD inputs through**

In `src/ui.rs`, add a second function:
```rust
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
    // Simple textual scale bar: show the world-units value of "10 % of view radius".
    egui::Area::new(egui::Id::new("scale"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(12.0, -12.0))
        .show(ctx, |ui| {
            let bar_a0 = h.camera_radius * 0.2; // ≈ width of scale bar in a₀
            let bar_nm = bar_a0 * 0.0529177;
            ui.label(format!("box: ±{:.1} a₀   |   bar ≈ {:.1} a₀ ({:.3} nm)", h.box_half, bar_a0, bar_nm));
        });
}
```

- [ ] **Step 2: Wire it into `main.rs`**

In `GpuState`, add:
```rust
    last_peak: f64,
    fps_accum: f32,
    fps_count: u32,
    fps_value: f32,
```
Initialize to `last_peak: 0.0, fps_accum: 0.0, fps_count: 0, fps_value: 0.0`.

In `render`, when rebake fires inside the egui closure, capture the peak:
```rust
            if rebake {
                let v = volume::bake(self.ui.n, self.ui.l, self.ui.m, self.ui.resolution);
                self.last_peak = v.peak;
                self.renderer.replace_volume(&self.device, &self.queue, &v);
                ...
            }
```
Also: set `self.last_peak` from the initial bake in `new` (use `initial.peak`).

After `dt` is computed, accumulate FPS:
```rust
        self.fps_accum += dt;
        self.fps_count += 1;
        if self.fps_accum >= 0.5 {
            self.fps_value = self.fps_count as f32 / self.fps_accum;
            self.fps_accum = 0.0;
            self.fps_count = 0;
        }
```

In the egui closure, after `ui::panel(...)`, also call:
```rust
            ui::hud(ctx, &ui::HudInputs {
                fps: self.fps_value,
                peak_psi_sq: self.last_peak,
                box_half: volume::box_extent(self.current_n),
                camera_radius: self.camera.radius,
            });
```

- [ ] **Step 3: Build, run, verify**

Run:
```pwsh
cargo run --release
```
Expected: FPS in top-right, density readout in bottom-right with scientific notation, scale info in bottom-left.

- [ ] **Step 4: Commit**

```pwsh
git add src/ui.rs src/main.rs
git commit -m "feat(ui): HUD overlays for FPS, peak density, and scale"
```

---

## Task 14: Screenshot (`S` key + button) → PNG

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add the keyboard binding**

In `window_event`'s keyboard handler, add an `S` case alongside the existing `F` case:
```rust
                        if code == winit::keyboard::KeyCode::KeyS {
                            gpu.ui.screenshot_requested = true;
                        }
```

- [ ] **Step 2: Implement framebuffer readback at end of `render`**

At the end of `render`, before `frame.present()`, replace the simple `self.queue.submit(...)` with a block that conditionally also reads back the surface texture into a CPU buffer:

```rust
        let do_shot = self.ui.screenshot_requested;
        self.ui.screenshot_requested = false;

        // Allocate readback buffer if needed.
        let readback = if do_shot {
            let bytes_per_pixel = 4;
            // Width must be aligned to 256 bytes for COPY_BUFFER_TO_TEXTURE/BUFFER copies.
            let unaligned_bpr = self.config.width * bytes_per_pixel;
            let align = 256;
            let padded_bpr = (unaligned_bpr + align - 1) / align * align;
            let size = padded_bpr as u64 * self.config.height as u64;
            let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("readback"),
                size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            // Copy the just-rendered surface texture.
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &frame.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buf,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bpr),
                        rows_per_image: Some(self.config.height),
                    },
                },
                wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
            );
            Some((buf, padded_bpr))
        } else {
            None
        };

        self.queue.submit(Some(encoder.finish()));

        if let Some((buf, padded_bpr)) = readback {
            let slice = buf.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            self.device.poll(wgpu::Maintain::Wait);
            let data = slice.get_mapped_range();
            let (w, h) = (self.config.width as usize, self.config.height as usize);
            let mut img: Vec<u8> = Vec::with_capacity(w * h * 4);
            for y in 0..h {
                let row_start = y * padded_bpr as usize;
                let row = &data[row_start..row_start + w * 4];
                // Surface is BGRA on most platforms; swap to RGBA.
                for px in row.chunks_exact(4) {
                    img.extend_from_slice(&[px[2], px[1], px[0], 255]);
                }
            }
            drop(data);
            buf.unmap();
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let fname = format!(
                "orbital_n{}l{}m{}_{}.png",
                self.current_n, self.current_l, self.current_m, ts
            );
            image::save_buffer(&fname, &img, w as u32, h as u32, image::ColorType::Rgba8)
                .expect("save png");
        }

        frame.present();
```

Note: if the surface format is RGBA-srgb (not BGRA), drop the channel swap. Verify by printing `self.config.format` once and adjusting if needed.

- [ ] **Step 3: Build, run, take a screenshot**

Run:
```pwsh
cargo run --release
```
Press `S` (or click Screenshot button). Expected: a `orbital_n3l2m1_<unix-ts>.png` appears in the project root showing the current view.

- [ ] **Step 4: Commit**

```pwsh
git add src/main.rs
git commit -m "feat: PNG screenshot on S key / button"
```

---

## Task 15: Polish — defaults, comment pass, final visual verification

**Files:**
- Modify: any of the above as needed.

- [ ] **Step 1: Eyeball default `k` and `exposure`**

Run:
```pwsh
cargo run --release
```
Cycle through presets: `1s`, `2p_z`, `3d_xz`, `4f_(z^3)`. At each, observe whether the cloud is well-exposed (cores visible but not blown out, tails faintly visible). If `k=5.0` and `exposure=1.0` produce something too dim or too bright across the range, adjust `UiState::default`. Acceptance criteria: at default `k`/exposure, every preset is visually intelligible — you can identify the orbital shape — without touching sliders.

- [ ] **Step 2: Visual smoke test of every UI control**

For each interaction below, verify the visible effect:
- Drag `n`, `l`, `m` sliders → cloud changes, no invalid-state crashes, no UI lag beyond a brief stutter on the rebake.
- Resolution dropdown 128 → 256 → 512 → confirm 512 stutters more (~300 ms) but doesn't deadlock the UI; revert to 256.
- All six colormaps → each renders without artifacts.
- `k` and exposure sliders → instant visual response, no rebake stutter.
- Auto-rotate checkbox → rotation starts/stops cleanly.
- Mouse drag → orbits. Scroll → zooms. `F` key + Fit button → both snap back.
- `S` key + Screenshot button → both produce a PNG.
- HUD: FPS visible top-right, density readout bottom-right updates on `(n,l,m)` change.

- [ ] **Step 3: Add the comment header to every module**

Each `.rs` file should start with a one-line `//!` doc comment naming its responsibility. Add any missing ones. (Most already have one from earlier tasks.)

- [ ] **Step 4: Run all tests one final time**

Run:
```pwsh
cargo test --release
```
Expected: all `physics::tests`, `volume::tests`, `volume::bake_tests`, `camera::tests` pass.

- [ ] **Step 5: Commit**

```pwsh
git add -A
git commit -m "chore: polish defaults and verify all interactions"
```

---

## Self-review checklist (run by the implementing agent)

After finishing Task 15, confirm against `spec.md`:

- [ ] §2 physics formulas implemented exactly as written; tests pass.
- [ ] §3 decision 1: real spherical harmonics (no complex pipeline anywhere).
- [ ] §3 decision 2: volume ray-march (no point sampling, no BVH, no spheres).
- [ ] §3 decision 3: adaptive box (`6·n²·a₀`), world-scale camera, F-key fit.
- [ ] §3 decision 4: emission + saturation (`1 - exp(-k·sum)`), no alpha compositing.
- [ ] §3 decision 5: per-orbital peak normalize; absolute peak shown in HUD.
- [ ] §3 decision 6: 256³ default, sync rebake, 128/256/512 dropdown.
- [ ] §3 decision 7: egui side panel, all knobs live.
- [ ] §3 decision 8: bounded-range `(n,l,m)` sliders.
- [ ] §3 decision 9: six colormaps via 1D LUTs, live dropdown.
- [ ] §3 decision 10: auto-rotate camera; no animated density.
- [ ] §7 HUD: FPS (top-right), scale info (bottom-left), density (bottom-right).
- [ ] §7 Screenshot: `S` key and button both work, PNG saved with `orbital_n{n}l{l}m{m}_{ts}.png` filename.
- [ ] §10 build-order: every step verified before moving on (the plan itself enforces this; the agent must not skip).
- [ ] §11 don'ts: no `LightingScaler`, no `pow(5, n)`, no `static bool built` pattern, no `calculateProbabilityFlow`.
