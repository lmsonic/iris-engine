use crate::renderer::color::Color;

use super::{
    bind_group::{AsBindGroup, BindGroupLayoutBuilder},
    image::Image,
    resources::ResourceHandle,
};

#[derive(Debug)]
pub struct StandardMaterial {
    pub diffuse_texture: ResourceHandle<Image>,
    pub diffuse_color: Color,
    pub normal_map: ResourceHandle<Image>,
    pub specular: f32,
    pub ior: f32,
    pub roughness: f32,
    pub ambient: Color,
}

impl AsBindGroup for StandardMaterial {
    type Data = Self;

    fn data(&self) -> Self::Data {
        todo!()
    }

    fn bindings(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> Vec<(u32, super::bind_group::OwnedBindingResource)> {
        todo!()
    }

    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        BindGroupLayoutBuilder::new(wgpu::ShaderStages::FRAGMENT)
            .texture_2d()
            .sampler(wgpu::SamplerBindingType::Filtering)
            .uniform()
            .texture_2d()
            .sampler(wgpu::SamplerBindingType::Filtering)
            .uniform()
            .uniform()
            .uniform()
            .uniform()
            .build(device)
    }
}
