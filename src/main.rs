mod camera;
mod physics;
mod render;
mod volume;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use camera::Camera;
use render::Renderer;
use volume::bake;

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    renderer: Renderer,
    camera: Camera,
    current_n: u32,
    current_l: u32,
    current_m: i32,
    mouse_down: bool,
    last_mouse: Option<(f64, f64)>,
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
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
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
        let initial = bake(3, 2, 1, 128);
        let renderer = Renderer::new(&device, &queue, config.format, &initial);
        let aspect = config.width as f32 / config.height as f32;
        let mut camera = Camera::new(2.0 * volume::box_extent(3) as f32, aspect);
        camera.fit(volume::box_extent(3) as f32);
        Self {
            surface, device, queue, config, window, renderer,
            camera,
            current_n: 3, current_l: 2, current_m: 1,
            mouse_down: false, last_mouse: None,
        }
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.config.width = w.max(1);
        self.config.height = h.max(1);
        self.surface.configure(&self.device, &self.config);
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
    }

    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f)
            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            _ => return,
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
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("run");
}
