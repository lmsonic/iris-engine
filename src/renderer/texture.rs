use crate::renderer::resources::get_max_mip_level_count;

use super::{compute, resources::load_texture};

use std::path::Path;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn new(path: impl AsRef<Path>, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let pathbuf = path.as_ref().to_owned();
        let (texture, view) = load_texture(path, device, queue)
            .unwrap_or_else(|_| panic!("Could not open {:?}", pathbuf.display()));

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: texture.mip_level_count() as f32,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        Self {
            texture,
            view,
            sampler,
        }
    }
    pub fn cubemap(
        paths: [impl AsRef<Path>; 6],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let mut cubemap_size = wgpu::Extent3d {
            width: 0,
            height: 0,
            depth_or_array_layers: 6,
        };
        let mut images = Vec::with_capacity(6);
        for (i, path) in paths.iter().enumerate() {
            let pathbuf = path.as_ref().to_owned();
            let image = image::open(path)
                .unwrap_or_else(|_| panic!("Could not open {:?}", pathbuf.display()));
            if i == 0 {
                cubemap_size.width = image.width();
                cubemap_size.height = image.height();
            } else {
                assert!(
                    cubemap_size.width == image.width() && cubemap_size.height == image.height(),
                    "Cubemap faces need to have the same size"
                );
            }
            images.push(image);
        }
        let mip_level_count = get_max_mip_level_count(cubemap_size.width, cubemap_size.height);
        let texture_descriptor = wgpu::TextureDescriptor {
            label: None,
            size: cubemap_size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let texture = device.create_texture(&texture_descriptor);
        for (layer, image) in images.iter().enumerate() {
            let destination = wgpu::ImageCopyTextureBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer as u32,
                },
                aspect: wgpu::TextureAspect::All,
            };
            let source = wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture.size().width),
                rows_per_image: Some(texture.size().height),
            };
            let data = image.as_rgba8().unwrap().as_raw();
            queue.write_texture(destination, data, source, texture.size());
            compute::generate_mipmaps(&texture, device, queue, layer as u32);
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(texture.format()),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(6),
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: mip_level_count as f32,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        Self {
            texture,
            view,
            sampler,
        }
    }
    pub fn depth(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let depth_texture_format = wgpu::TextureFormat::Depth24Plus;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth_texture_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[depth_texture_format],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Depth Texture View"),
            format: Some(depth_texture_format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
        }
    }
}
