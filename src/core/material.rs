use crate::{core::buffer::UniformBuffer, renderer::color::Color};

use super::{
    bind_group::{AsBindGroup, BindGroupLayoutBuilder, OwnedBindingResource},
    image::Image,
    resources::{ResourceHandle, ResourceManager},
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
        queue: &wgpu::Queue,
        resources: &ResourceManager,
    ) -> Vec<OwnedBindingResource> {
        let diffuse_texture = resources.load_resource(&self.diffuse_texture).unwrap();
        let normal_texture = resources.load_resource(&self.normal_map).unwrap();
        let diffuse_view = diffuse_texture.to_gpu(device, queue);
        let normal_view = normal_texture.to_gpu(device, queue);
        vec![
            OwnedBindingResource::TextureView(diffuse_view.texture_view),
            OwnedBindingResource::Sampler(diffuse_view.sampler),
            OwnedBindingResource::Buffer(UniformBuffer::new(self.diffuse_color, device).buffer),
            OwnedBindingResource::TextureView(normal_view.texture_view),
            OwnedBindingResource::Sampler(normal_view.sampler),
            OwnedBindingResource::Buffer(UniformBuffer::new(self.specular, device).buffer),
            OwnedBindingResource::Buffer(UniformBuffer::new(self.ior, device).buffer),
            OwnedBindingResource::Buffer(UniformBuffer::new(self.roughness, device).buffer),
            OwnedBindingResource::Buffer(UniformBuffer::new(self.ambient, device).buffer),
        ]
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
