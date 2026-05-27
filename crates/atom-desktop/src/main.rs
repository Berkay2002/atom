mod camera;
mod colormaps;
mod input;
mod render;
mod runtime;
mod scene_bake;
mod screenshot;
mod ui;
mod ui_tokens;
mod ui_widgets;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use input::InputRouter;
use runtime::DesktopRuntime;

#[derive(Default)]
struct App {
    runtime: Option<DesktopRuntime>,
    input_router: InputRouter,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.runtime.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("atom"))
                .expect("window"),
        );
        let runtime = pollster::block_on(DesktopRuntime::new(window));
        self.runtime = Some(runtime);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };
        if runtime.egui_on_window_event(&event) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => runtime.on_resized(size.width, size.height),
            WindowEvent::RedrawRequested => {
                runtime.render_frame();
                runtime.window.request_redraw();
            }
            _ => runtime.on_window_event(&event, &mut self.input_router),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("run");
}
