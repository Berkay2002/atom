//! egui side panel: bounded (n, l, m) sliders. Returns `true` from `panel()` if
//! a parameter that requires a volume rebake changed.

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
            resolution: 128,
            k: 5.0,
            exposure: 1.0,
            auto_rotate: false,
            colormap_index: 0,
            fit_requested: false,
            screenshot_requested: false,
        }
    }
}

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
        let cmap_names: Vec<&str> = crate::colormaps::ALL.iter().map(|(n, _)| *n).collect();
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
    needs_rebake
}
