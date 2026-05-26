//! Orbit camera around the origin. World units = a₀.

use glam::{Mat4, Vec3};

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
        c.orbit(0.0, -10_000.0);
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
