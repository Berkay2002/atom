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

#[allow(dead_code)]
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

// 4 reference stops linearly interpolated into a 256-entry LUT.
// Used as the initial inferno-ish LUT; Task 11 replaces with full set.
const DEFAULT_LUT_STOPS: &[[u8; 3]] = &[
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
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
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
            multiview_mask: None,
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
        let (lut_tex, lut_view) = upload_lut_texture(device, queue, DEFAULT_LUT_STOPS);
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

    #[allow(dead_code)]
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

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("raymarch"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
