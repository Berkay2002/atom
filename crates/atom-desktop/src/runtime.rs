use std::sync::Arc;

use atom_core::scene::{Orbital, Scene};
use atom_core::volume;
use winit::window::Window;

use crate::camera::Camera;
use crate::input::InputRouter;
use crate::render::Renderer;
use crate::scene_bake::SceneBakeState;
use crate::screenshot::ScreenshotReadback;
use crate::ui;
use crate::ui::UiState;

pub struct DesktopRuntime {
    pub window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    camera: Camera,
    scene_bake: SceneBakeState,
    current_colormap: usize,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    ui: UiState,
    last_frame: std::time::Instant,
    fps_accum: f32,
    fps_count: u32,
    fps_value: f32,
}

impl DesktopRuntime {
    pub async fn new(window: Arc<Window>) -> Self {
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
                required_features: wgpu::Features::FLOAT32_FILTERABLE,
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
        let initial =
            volume::bake_scene(&Scene::single_hydrogen(Orbital { n: 3, l: 2, m: 1 }), 256);
        let scene_bake = SceneBakeState::new(1, Orbital { n: 3, l: 2, m: 1 }, false, 256, &initial);
        let renderer = {
            let mut r = Renderer::new(&device, &queue, config.format, &initial);
            r.replace_lut(&device, &queue, crate::colormaps::ALL[0].1);
            r
        };
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
            window,
            surface,
            device,
            queue,
            config,
            renderer,
            camera,
            scene_bake,
            current_colormap: 0,
            egui_ctx,
            egui_state,
            egui_renderer,
            ui,
            last_frame,
            fps_accum: 0.0,
            fps_count: 0,
            fps_value: 0.0,
        }
    }

    pub fn on_window_event(
        &mut self,
        event: &winit::event::WindowEvent,
        input_router: &mut InputRouter,
    ) {
        input_router.handle_window_event(
            event,
            &mut self.camera,
            &self.scene_bake,
            &mut self.ui,
            self.egui_ctx.egui_wants_keyboard_input(),
        );
    }

    pub fn on_resized(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
    }

    pub fn render_frame(&mut self) {
        if self.ui.fit_requested {
            self.camera.fit(self.scene_bake.current_half_extent());
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
        let full_output = self.egui_ctx.run_ui(raw_input, |ctx| {
            let rebake_from_panel = ui::panel(ctx, &mut self.ui);
            let rebake_from_hud = ui::hud(
                ctx,
                &ui::HudInputs {
                    fps: self.fps_value,
                    peak_psi_sq: self.scene_bake.last_peak(),
                    box_half: self.scene_bake.current_half_extent() as f64,
                    camera_radius: self.camera.radius,
                },
                &mut self.ui,
            );
            rebake_requested = rebake_from_panel || rebake_from_hud;
        });
        if self.scene_bake.should_rebake(&self.ui, rebake_requested) {
            let scene = SceneBakeState::scene_from_ui(&self.ui);
            let v = volume::bake_scene(&scene, SceneBakeState::resolution_from_ui(&self.ui));
            self.renderer.replace_volume(&self.device, &self.queue, &v);
            self.scene_bake.apply_bake(&self.ui, &v);
            self.camera.fit(self.scene_bake.current_half_extent());
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
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let half = self.scene_bake.current_half_extent();
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
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        self.renderer.draw(&mut encoder, &view);

        let do_shot = self.ui.screenshot_requested;
        self.ui.screenshot_requested = false;
        let readback = do_shot.then(|| {
            ScreenshotReadback::schedule(
                &self.device,
                &mut encoder,
                &frame.texture,
                self.config.width,
                self.config.height,
                self.config.format,
            )
        });

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
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        self.queue.submit(Some(encoder.finish()));

        if let Some(readback) = readback {
            readback.save(&self.device, self.scene_bake.screenshot_fields());
        }

        frame.present();
    }

    pub fn egui_on_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.egui_state
            .on_window_event(&self.window, event)
            .consumed
    }
}
