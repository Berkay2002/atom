use crate::camera::Camera;
use crate::scene_bake::SceneBakeState;
use crate::ui::UiState;
use winit::event::ElementState;
use winit::event::{MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutCommand {
    FitCamera,
    RequestScreenshot,
    ToggleHud,
    ToggleAutoRotate,
}

pub fn map_shortcut_command(
    state: ElementState,
    key: PhysicalKey,
    egui_wants_keyboard_input: bool,
) -> Option<ShortcutCommand> {
    if state != ElementState::Pressed || egui_wants_keyboard_input {
        return None;
    }
    match key {
        PhysicalKey::Code(KeyCode::KeyF) => Some(ShortcutCommand::FitCamera),
        PhysicalKey::Code(KeyCode::KeyS) => Some(ShortcutCommand::RequestScreenshot),
        PhysicalKey::Code(KeyCode::KeyH) => Some(ShortcutCommand::ToggleHud),
        PhysicalKey::Code(KeyCode::Space) => Some(ShortcutCommand::ToggleAutoRotate),
        _ => None,
    }
}

#[derive(Default)]
pub struct InputRouter {
    mouse_down: bool,
    last_mouse: Option<(f64, f64)>,
}

impl InputRouter {
    pub fn handle_window_event(
        &mut self,
        event: &WindowEvent,
        camera: &mut Camera,
        scene_bake: &SceneBakeState,
        ui: &mut UiState,
        egui_wants_keyboard_input: bool,
    ) {
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    self.mouse_down = *state == ElementState::Pressed;
                    if !self.mouse_down {
                        self.last_mouse = None;
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                if self.mouse_down {
                    if let Some((px, py)) = self.last_mouse {
                        let dx = (x - px) as f32;
                        let dy = (y - py) as f32;
                        camera.orbit(dx, dy);
                    }
                }
                self.last_mouse = Some((x, y));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 50.0,
                };
                let factor = (1.0 - scroll * 0.1).clamp(0.5, 2.0);
                camera.zoom(factor);
            }
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if let Some(command) = map_shortcut_command(
                    key_event.state,
                    key_event.physical_key,
                    egui_wants_keyboard_input,
                ) {
                    apply_shortcut(command, camera, scene_bake, ui);
                }
            }
            _ => {}
        }
    }
}

fn apply_shortcut(
    command: ShortcutCommand,
    camera: &mut Camera,
    scene_bake: &SceneBakeState,
    ui: &mut UiState,
) {
    match command {
        ShortcutCommand::FitCamera => camera.fit(scene_bake.current_half_extent()),
        ShortcutCommand::RequestScreenshot => ui.screenshot_requested = true,
        ShortcutCommand::ToggleHud => ui.hud_visible = !ui.hud_visible,
        ShortcutCommand::ToggleAutoRotate => ui.auto_rotate = !ui.auto_rotate,
    }
}

#[cfg(test)]
mod tests {
    use super::{map_shortcut_command, ShortcutCommand};
    use winit::event::ElementState;
    use winit::keyboard::{KeyCode, PhysicalKey};

    #[test]
    fn maps_shortcuts_when_pressed_and_not_captured() {
        assert_eq!(
            map_shortcut_command(
                ElementState::Pressed,
                PhysicalKey::Code(KeyCode::KeyF),
                false
            ),
            Some(ShortcutCommand::FitCamera)
        );
        assert_eq!(
            map_shortcut_command(
                ElementState::Pressed,
                PhysicalKey::Code(KeyCode::KeyS),
                false
            ),
            Some(ShortcutCommand::RequestScreenshot)
        );
        assert_eq!(
            map_shortcut_command(
                ElementState::Pressed,
                PhysicalKey::Code(KeyCode::KeyH),
                false
            ),
            Some(ShortcutCommand::ToggleHud)
        );
        assert_eq!(
            map_shortcut_command(
                ElementState::Pressed,
                PhysicalKey::Code(KeyCode::Space),
                false
            ),
            Some(ShortcutCommand::ToggleAutoRotate)
        );
    }

    #[test]
    fn ignores_shortcuts_when_egui_wants_keyboard_input() {
        assert_eq!(
            map_shortcut_command(
                ElementState::Pressed,
                PhysicalKey::Code(KeyCode::KeyF),
                true
            ),
            None
        );
    }

    #[test]
    fn ignores_non_pressed_and_non_mapped_keys() {
        assert_eq!(
            map_shortcut_command(
                ElementState::Released,
                PhysicalKey::Code(KeyCode::KeyF),
                false
            ),
            None
        );
        assert_eq!(
            map_shortcut_command(
                ElementState::Pressed,
                PhysicalKey::Code(KeyCode::KeyA),
                false
            ),
            None
        );
    }
}
