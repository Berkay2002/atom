use atom_core::scene::{Atom, ElementId, Orbital, Scene, View};
use atom_core::volume::Volume;

use crate::ui::UiState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BakeSelection {
    element_z: u32,
    n: u32,
    l: u32,
    m: i32,
    use_bare_z: bool,
    resolution: usize,
}

impl BakeSelection {
    fn from_ui(ui: &UiState) -> Self {
        Self {
            element_z: ui.element_z,
            n: ui.n,
            l: ui.l,
            m: ui.m,
            use_bare_z: ui.use_bare_z,
            resolution: ui.resolution,
        }
    }
}

pub struct SceneBakeState {
    selection: BakeSelection,
    current_half_extent: f32,
    last_peak: f64,
}

impl SceneBakeState {
    pub fn new(
        element_z: u32,
        orbital: Orbital,
        use_bare_z: bool,
        resolution: usize,
        volume: &Volume,
    ) -> Self {
        Self {
            selection: BakeSelection {
                element_z,
                n: orbital.n,
                l: orbital.l,
                m: orbital.m,
                use_bare_z,
                resolution,
            },
            current_half_extent: volume.half_extent as f32,
            last_peak: volume.peak,
        }
    }

    pub fn should_rebake(&self, ui: &UiState, ui_requested: bool) -> bool {
        ui_requested && self.selection != BakeSelection::from_ui(ui)
    }

    pub fn scene_from_ui(ui: &UiState) -> Scene {
        Scene {
            atoms: vec![Atom {
                element: ElementId(ui.element_z),
                position: [0.0, 0.0, 0.0],
                orbital: Orbital {
                    n: ui.n,
                    l: ui.l,
                    m: ui.m,
                },
            }],
            view: View {
                use_bare_z: ui.use_bare_z,
                ..View::default()
            },
        }
    }

    pub fn resolution_from_ui(ui: &UiState) -> usize {
        ui.resolution
    }

    pub fn apply_bake(&mut self, ui: &UiState, volume: &Volume) {
        self.selection = BakeSelection::from_ui(ui);
        self.current_half_extent = volume.half_extent as f32;
        self.last_peak = volume.peak;
    }

    pub fn current_half_extent(&self) -> f32 {
        self.current_half_extent
    }

    pub fn last_peak(&self) -> f64 {
        self.last_peak
    }

    pub fn screenshot_fields(&self) -> (u32, u32, u32, i32) {
        (
            self.selection.element_z,
            self.selection.n,
            self.selection.l,
            self.selection.m,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SceneBakeState;
    use atom_core::scene::Orbital;
    use atom_core::volume::Volume;

    use crate::ui::UiState;

    fn volume(half_extent: f64, peak: f64) -> Volume {
        Volume {
            data: vec![0.0],
            res: 1,
            half_extent,
            peak,
        }
    }

    #[test]
    fn rebake_only_when_ui_signals_and_selection_changes() {
        let initial = volume(2.5, 7.0);
        let mut ui = UiState::default();
        ui.element_z = 6;
        ui.n = 2;
        ui.l = 1;
        ui.m = 0;
        ui.use_bare_z = false;
        ui.resolution = 256;

        let state = SceneBakeState::new(
            ui.element_z,
            Orbital {
                n: ui.n,
                l: ui.l,
                m: ui.m,
            },
            ui.use_bare_z,
            ui.resolution,
            &initial,
        );

        assert!(!state.should_rebake(&ui, false));
        assert!(!state.should_rebake(&ui, true));

        ui.k = 9.0;
        ui.exposure = 2.0;
        ui.colormap_index = 1;
        assert!(!state.should_rebake(&ui, true));

        ui.n = 3;
        assert!(state.should_rebake(&ui, true));
        assert!(!state.should_rebake(&ui, false));
    }

    #[test]
    fn apply_bake_updates_selection_peak_and_half_extent() {
        let initial = volume(10.0, 1.0);
        let mut ui = UiState::default();
        let mut state = SceneBakeState::new(
            1,
            Orbital { n: 3, l: 2, m: 1 },
            false,
            256,
            &initial,
        );

        ui.element_z = 8;
        ui.n = 2;
        ui.l = 1;
        ui.m = -1;
        ui.use_bare_z = true;
        ui.resolution = 128;

        let baked = volume(4.0, 3.5);
        state.apply_bake(&ui, &baked);

        assert_eq!(state.current_half_extent(), 4.0);
        assert_eq!(state.last_peak(), 3.5);
        assert_eq!(state.screenshot_fields(), (8, 2, 1, -1));
    }
}
