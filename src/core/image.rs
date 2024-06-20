use image::{DynamicImage, ImageError};
use wgpu::{util::DeviceExt, Extent3d};

use crate::{core::compute::ComputePipelineBuilder, renderer::resources::get_max_mip_level_count};

use super::{bind_group::BindGroupLayoutBuilder, resources::Resource};

use std::path::Path;

#[derive(Debug)]
pub struct Image {
    pub image: DynamicImage,
    pub texture_descriptor: wgpu::TextureDescriptor<'static>,
    pub sampler: wgpu::SamplerDescriptor<'static>,
    pub texture_view_descriptor: Option<wgpu::TextureViewDescriptor<'static>>,
}

impl Resource for Image {}

impl Default for Image {
    fn default() -> Self {
        Self {
            image: DynamicImage::default(),
            texture_descriptor: wgpu::TextureDescriptor {
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                format: wgpu::TextureFormat::Rgba8Unorm,
                dimension: wgpu::TextureDimension::D2,
                label: None,
                mip_level_count: 1,
                sample_count: 1,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            sampler: wgpu::SamplerDescriptor::default(),
            texture_view_descriptor: Option::default(),
        }
    }
}

#[derive(Debug)]
pub struct GpuImage {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub texture_format: wgpu::TextureFormat,
    pub sampler: wgpu::Sampler,
    pub size: Extent3d,
    pub mip_level_count: u32,
}

impl GpuImage {
    pub fn generate_mipmaps(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, layer: u32) {
        // Create mip views and sizes
        let mut mip_sizes = vec![self.texture.size()];
        let mut mip_views = vec![];
        let mip_level_count = self.texture.mip_level_count();
        for level in 0..mip_level_count {
            mip_views.push(self.texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("mip view: {level}")),
                format: Some(self.texture_format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: level,
                mip_level_count: Some(1),
                base_array_layer: layer,
                array_layer_count: Some(self.texture.depth_or_array_layers()),
            }));
            if level > 0 {
                let previous_size = mip_sizes[level as usize - 1];
                mip_sizes.push(Extent3d {
                    width: previous_size.width / 2,
                    height: previous_size.height / 2,
                    depth_or_array_layers: previous_size.depth_or_array_layers / 2,
                });
            }
        }

        let bind_group_layout = BindGroupLayoutBuilder::new(wgpu::ShaderStages::COMPUTE)
            .texture_2d()
            .storage_texture(
                wgpu::StorageTextureAccess::WriteOnly,
                wgpu::TextureFormat::Rgba8Unorm,
                wgpu::TextureViewDimension::D2,
            )
            .build(device);

        // Create bind groups in advance because of rust borrow rules
        let mut bind_groups = vec![];
        for level in 1..mip_level_count {
            bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &mip_views[level as usize - 1],
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&mip_views[level as usize]),
                    },
                ],
            }));
        }

        let shader = wgpu::include_wgsl!("shaders/mipmap_generation.wgsl");

        let compute_pipeline = ComputePipelineBuilder::new(shader)
            .add_bind_group(&bind_group_layout)
            .build(device, "compute_mip_map");

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&compute_pipeline);

        for level in 1..mip_level_count {
            // We write to each mip level using the previous level
            let level = level as usize;
            compute_pass.set_bind_group(0, &bind_groups[level - 1], &[]);
            let invocation_count_x = mip_sizes[level - 1].width;
            let invocation_count_y = mip_sizes[level - 1].height;
            let workgroup_size_per_dim = 8;
            // This ceils invocation_count / workgroup_size
            let workgroup_count_x =
                (invocation_count_x + workgroup_size_per_dim - 1) / workgroup_size_per_dim;
            let workgroup_count_y =
                (invocation_count_y + workgroup_size_per_dim - 1) / workgroup_size_per_dim;
            compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }

        drop(compute_pass);

        let command = encoder.finish();

        queue.submit([command]);
    }
}

impl Image {
    pub fn new(image: DynamicImage) -> Self {
        let mut image = Self {
            image,
            ..Default::default()
        };
        image.texture_descriptor.dimension = wgpu::TextureDimension::D2;
        image.texture_descriptor.size = Extent3d {
            width: image.image.width(),
            height: image.image.height(),
            depth_or_array_layers: 1,
        };
        image.texture_descriptor.format = wgpu::TextureFormat::Rgba8Unorm;
        let mip_level_count = get_max_mip_level_count(image.image.width(), image.image.height());
        image.texture_descriptor.mip_level_count = mip_level_count;
        image
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ImageError> {
        let image = image::open(&path)?;

        Ok(Self::new(image))
    }
    pub fn to_gpu(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> GpuImage {
        let binding = self.image.as_rgb8().unwrap();
        let data = binding.as_raw();
        let texture = device.create_texture_with_data(
            queue,
            &self.texture_descriptor,
            wgpu::util::TextureDataOrder::MipMajor,
            data,
        );

        let size = self.texture_descriptor.size;
        let texture_view_descriptor = self.texture_view_descriptor.clone().unwrap_or_default();

        let texture_view = texture.create_view(&texture_view_descriptor);
        let sampler = device.create_sampler(&self.sampler);

        let mut image = GpuImage {
            texture,
            texture_view,
            texture_format: self.texture_descriptor.format,
            sampler,
            size,
            mip_level_count: self.texture_descriptor.mip_level_count,
        };
        if image.texture.dimension() == wgpu::TextureDimension::D2 {
            for layer in 0..=size.depth_or_array_layers {
                image.generate_mipmaps(device, queue, layer);
            }
        }
        image
    }
}
