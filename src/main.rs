mod camera;
mod colormaps;
mod physics;
mod render;
mod ui;
mod volume;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use camera::Camera;
use render::Renderer;
use ui::UiState;
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
    current_colormap: usize,
    mouse_down: bool,
    last_mouse: Option<(f64, f64)>,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    ui: UiState,
    last_frame: std::time::Instant,
    last_peak: f64,
    fps_accum: f32,
    fps_count: u32,
    fps_value: f32,
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
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
        let last_peak = initial.peak;
        let renderer = Renderer::new(&device, &queue, config.format, &initial);
        let aspect = config.width as f32 / config.height as f32;
        let mut camera = Camera::new(2.0 * volume::box_extent(3) as f32, aspect);
        camera.fit(volume::box_extent(3) as f32);
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            config.format,
            egui_wgpu::RendererOptions::default(),
        );
        let ui = UiState::default();
        let last_frame = std::time::Instant::now();
        Self {
            surface, device, queue, config, window, renderer,
            camera,
            current_n: 3, current_l: 2, current_m: 1,
            current_colormap: 0,
            mouse_down: false, last_mouse: None,
            egui_ctx, egui_state, egui_renderer, ui,
            last_frame,
            last_peak,
            fps_accum: 0.0,
            fps_count: 0,
            fps_value: 0.0,
        }
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.config.width = w.max(1);
        self.config.height = h.max(1);
        self.surface.configure(&self.device, &self.config);
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
    }

    fn render(&mut self) {
        if self.ui.fit_requested {
            self.camera.fit(volume::box_extent(self.current_n) as f32);
            self.ui.fit_requested = false;
        }

        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.fps_accum += dt;
        self.fps_count += 1;
        if self.fps_accum >= 0.5 {
            self.fps_value = self.fps_count as f32 / self.fps_accum;
            self.fps_accum = 0.0;
            self.fps_count = 0;
        }
        if self.ui.auto_rotate {
            self.camera.azimuth += 0.2 * dt;
        }

        let raw_input = self.egui_state.take_egui_input(&self.window);
        let mut rebake_requested = false;
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            rebake_requested = ui::panel(ctx, &mut self.ui);
            ui::hud(ctx, &ui::HudInputs {
                fps: self.fps_value,
                peak_psi_sq: self.last_peak,
                box_half: volume::box_extent(self.current_n),
                camera_radius: self.camera.radius,
            });
        });
        if rebake_requested {
            let v = volume::bake(self.ui.n, self.ui.l, self.ui.m, self.ui.resolution);
            self.last_peak = v.peak;
            self.renderer.replace_volume(&self.device, &self.queue, &v);
            self.current_n = self.ui.n;
            self.current_l = self.ui.l;
            self.current_m = self.ui.m;
        }
        if self.ui.colormap_index != self.current_colormap {
            let stops = crate::colormaps::ALL[self.ui.colormap_index].1;
            self.renderer.replace_lut(&self.device, &self.queue, stops);
            self.current_colormap = self.ui.colormap_index;
        }
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
            self.ui.k,
            self.ui.exposure,
            self.renderer.res as f32,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });

        self.renderer.draw(&mut encoder, &view);

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
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.egui_renderer
                .render(&mut pass.forget_lifetime(), &paint_jobs, &screen);
        }
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        let do_shot = self.ui.screenshot_requested;
        self.ui.screenshot_requested = false;

        let readback = if do_shot {
            let bytes_per_pixel = 4u32;
            let unaligned_bpr = self.config.width * bytes_per_pixel;
            let align = 256u32;
            let padded_bpr = ((unaligned_bpr + align - 1) / align) * align;
            let size = padded_bpr as u64 * self.config.height as u64;
            let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("readback"),
                size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
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
            let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
            let data = slice.get_mapped_range();
            let (w, h) = (self.config.width as usize, self.config.height as usize);
            let mut img: Vec<u8> = Vec::with_capacity(w * h * 4);
            let is_bgra = matches!(
                self.config.format,
                wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
            );
            for y in 0..h {
                let row_start = y * padded_bpr as usize;
                let row = &data[row_start..row_start + w * 4];
                for px in row.chunks_exact(4) {
                    let (r, g, b) = if is_bgra {
                        (px[2], px[1], px[0])
                    } else {
                        (px[0], px[1], px[2])
                    };
                    img.extend_from_slice(&[r, g, b, 255]);
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
            image::save_buffer(
                &fname,
                &img,
                w as u32,
                h as u32,
                image::ExtendedColorType::Rgba8,
            )
            .expect("save png");
        }

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
        let response = gpu.egui_state.on_window_event(&gpu.window, &event);
        if response.consumed {
            return;
        }
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
                        if code == winit::keyboard::KeyCode::KeyS {
                            gpu.ui.screenshot_requested = true;
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
