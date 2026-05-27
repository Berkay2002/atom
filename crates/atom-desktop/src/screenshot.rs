use std::time::{SystemTime, UNIX_EPOCH};

pub struct ScreenshotReadback {
    buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    padded_bpr: u32,
    is_bgra: bool,
}

impl ScreenshotReadback {
    pub fn schedule(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame_texture: &wgpu::Texture,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let bytes_per_pixel = 4u32;
        let unaligned_bpr = width * bytes_per_pixel;
        let align = 256u32;
        let padded_bpr = unaligned_bpr.div_ceil(align) * align;
        let size = padded_bpr as u64 * height as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: frame_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bpr),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let is_bgra = matches!(
            format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        );
        Self {
            buffer,
            width,
            height,
            padded_bpr,
            is_bgra,
        }
    }

    pub fn save(self, device: &wgpu::Device, screenshot_fields: (u32, u32, u32, i32)) {
        let slice = self.buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let data = slice.get_mapped_range();

        let (w, h) = (self.width as usize, self.height as usize);
        let mut img: Vec<u8> = Vec::with_capacity(w * h * 4);
        for y in 0..h {
            let row_start = y * self.padded_bpr as usize;
            let row = &data[row_start..row_start + w * 4];
            for px in row.chunks_exact(4) {
                let (r, g, b) = if self.is_bgra {
                    (px[2], px[1], px[0])
                } else {
                    (px[0], px[1], px[2])
                };
                img.extend_from_slice(&[r, g, b, 255]);
            }
        }
        drop(data);
        self.buffer.unmap();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let filename = screenshot_filename(screenshot_fields, timestamp);
        image::save_buffer(
            &filename,
            &img,
            self.width,
            self.height,
            image::ExtendedColorType::Rgba8,
        )
        .expect("save png");
    }
}

pub fn screenshot_filename(fields: (u32, u32, u32, i32), timestamp: u64) -> String {
    let (z, n, l, m) = fields;
    format!("orbital_z{z}_n{n}l{l}m{m}_{timestamp}.png")
}

#[cfg(test)]
mod tests {
    use super::screenshot_filename;

    #[test]
    fn screenshot_filename_includes_element_and_orbital_identifiers() {
        assert_eq!(
            screenshot_filename((8, 2, 1, -1), 12345),
            "orbital_z8_n2l1m-1_12345.png"
        );
    }
}
