use egui::Ui;
use image::{DynamicImage, ImageBuffer};
use wgpu::{core::device::queue, include_wgsl, ShaderModuleDescriptor};

use super::{
    bind_group::{BindGroup, BindGroupBuilder},
    buffer::UniformBuffer,
    color::Color,
    gui::{color_edit, float_edit},
    render_pipeline::RenderPipelineBuilder,
    texture::Texture,
};

pub trait Material<'a> {
    fn shader() -> ShaderModuleDescriptor<'a>;
    fn bind_group(&self) -> &BindGroup;
}
pub struct UnlitMaterial {
    pub diffuse_texture: Texture,
    pub diffuse_color: UniformBuffer<Color>,
    pub bind_group: BindGroup,
}

impl UnlitMaterial {
    pub fn gui(&mut self, ui: &mut Ui, queue: &wgpu::Queue) {
        if color_edit(ui, &mut self.diffuse_color.data, "Diffuse Color") {
            self.diffuse_color.update(queue);
        }
    }
}

#[derive(Default)]
pub struct UnlitMaterialBuilder {
    diffuse_texture: Option<Texture>,
    diffuse_color: Option<Color>,
}

impl UnlitMaterialBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn diffuse_texture(self, diffuse_texture: Texture) -> Self {
        Self {
            diffuse_texture: Some(diffuse_texture),
            ..self
        }
    }
    pub fn diffuse_color(self, diffuse_color: Color) -> Self {
        Self {
            diffuse_color: Some(diffuse_color),
            ..self
        }
    }
    pub fn build(self, device: &wgpu::Device, queue: &wgpu::Queue) -> UnlitMaterial {
        let diffuse_color = self.diffuse_color.unwrap_or(Color::WHITE);
        let diffuse_color = UniformBuffer::new(diffuse_color, device);
        let diffuse_texture = self.diffuse_texture.unwrap_or_else(|| {
            let default_white_image: DynamicImage =
                ImageBuffer::from_pixel(2, 2, image::Rgba([255_u8, 255_u8, 255_u8, 255_u8])).into();
            Texture::new(default_white_image, device, queue)
        });
        let bind_group = BindGroupBuilder::new()
            .texture(&diffuse_texture)
            .uniform(&diffuse_color.buffer)
            .build(device);
        UnlitMaterial {
            diffuse_texture,
            diffuse_color,
            bind_group,
        }
    }
}

pub struct LitMaterial {
    pub diffuse_texture: Texture,
    pub diffuse_color: UniformBuffer<Color>,
    pub normal_map: Texture,
    pub specular_color: UniformBuffer<Color>,
    pub specular_exponent: UniformBuffer<f32>,
    pub ambient: UniformBuffer<Color>,
    pub bind_group: BindGroup,
}

impl LitMaterial {
    pub fn gui(&mut self, ui: &mut Ui, queue: &wgpu::Queue) {
        if color_edit(ui, &mut self.diffuse_color.data, "Diffuse Color") {
            self.diffuse_color.update(queue);
        }
        if color_edit(ui, &mut self.specular_color.data, "Specular Color") {
            self.specular_color.update(queue);
        }

        if float_edit(
            ui,
            &mut self.specular_exponent.data,
            "Specular Exponent",
            0.0..=1000.0,
        ) {
            self.specular_exponent.update(queue);
        }
        if color_edit(ui, &mut self.ambient.data, "Ambient Color") {
            self.ambient.update(queue);
        }
    }
}

#[derive(Default)]
pub struct LitMaterialBuilder {
    diffuse_texture: Option<Texture>,
    diffuse_color: Option<Color>,
    normal_map: Option<Texture>,
    specular_color: Option<Color>,
    specular_exponent: Option<f32>,
    ambient: Option<Color>,
}

impl LitMaterialBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn diffuse_texture(self, diffuse_texture: Texture) -> Self {
        Self {
            diffuse_texture: Some(diffuse_texture),
            ..self
        }
    }
    pub fn diffuse_color(self, diffuse_color: Color) -> Self {
        Self {
            diffuse_color: Some(diffuse_color),
            ..self
        }
    }
    pub fn normal_texture(self, normal_texture: Texture) -> Self {
        Self {
            normal_map: Some(normal_texture),
            ..self
        }
    }
    pub fn specular_color(self, specular_color: Color) -> Self {
        Self {
            specular_color: Some(specular_color),
            ..self
        }
    }
    pub fn specular_exponent(self, specular_exponent: f32) -> Self {
        Self {
            specular_exponent: Some(specular_exponent),
            ..self
        }
    }
    pub fn ambient(self, ambient: Color) -> Self {
        Self {
            ambient: Some(ambient),
            ..self
        }
    }
    pub fn build(self, device: &wgpu::Device, queue: &wgpu::Queue) -> LitMaterial {
        let diffuse_color = UniformBuffer::new(self.diffuse_color.unwrap_or(Color::WHITE), device);
        let diffuse_texture = self.diffuse_texture.unwrap_or_else(|| {
            let default_white_image: DynamicImage =
                ImageBuffer::from_pixel(2, 2, image::Rgba([255_u8, 255_u8, 255_u8, 255_u8])).into();
            Texture::new(default_white_image, device, queue)
        });
        let specular_color =
            UniformBuffer::new(self.specular_color.unwrap_or(Color::WHITE), device);

        let specular_exponent = UniformBuffer::new(self.specular_exponent.unwrap_or(100.0), device);
        let normal_map = self.normal_map.unwrap_or_else(|| {
            let default_normal_map: DynamicImage =
                ImageBuffer::from_pixel(2, 2, image::Rgba([127_u8, 127_u8, 127_u8, 255_u8])).into();
            Texture::new(default_normal_map, device, queue)
        });
        let ambient = UniformBuffer::new(self.ambient.unwrap_or(Color::WHITE * 0.1), device);
        let bind_group = BindGroupBuilder::new()
            .texture(&diffuse_texture)
            .uniform(&diffuse_color.buffer)
            .texture(&normal_map)
            .uniform(&specular_color.buffer)
            .uniform(&specular_exponent.buffer)
            .uniform(&ambient.buffer)
            .build(device);
        LitMaterial {
            diffuse_texture,
            diffuse_color,
            bind_group,
            normal_map,
            specular_color,
            specular_exponent,
            ambient,
        }
    }
}

pub struct PbrMaterial {
    pub diffuse_texture: Texture,
    pub diffuse_color: UniformBuffer<Color>,
    pub normal_map: Texture,
    pub specular: UniformBuffer<f32>,
    pub ior: UniformBuffer<f32>,
    pub roughness: UniformBuffer<f32>,
    pub ambient: UniformBuffer<Color>,
    pub bind_group: BindGroup,
}

impl PbrMaterial {
    pub fn gui(&mut self, ui: &mut Ui, queue: &wgpu::Queue) {
        if color_edit(ui, &mut self.diffuse_color.data, "Diffuse Color") {
            self.diffuse_color.update(queue);
        }
        if float_edit(ui, &mut self.specular.data, "Specular Intensity", 0.0..=1.0) {
            self.specular.update(queue);
        }
        if float_edit(ui, &mut self.ior.data, "Index of Refraction", 0.5..=3.0) {
            self.ior.update(queue);
        }
        if float_edit(ui, &mut self.roughness.data, "Roughness", 0.0..=1.0) {
            self.roughness.update(queue);
        }
        if color_edit(ui, &mut self.ambient.data, "Ambient Color") {
            self.ambient.update(queue);
        }
    }
}
#[derive(Default)]
pub struct PbrMaterialBuilder {
    diffuse_texture: Option<Texture>,
    diffuse_color: Option<Color>,
    normal_map: Option<Texture>,
    specular: Option<f32>,
    ior: Option<f32>,
    roughness: Option<f32>,
    ambient: Option<Color>,
}

impl PbrMaterialBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn diffuse_texture(self, diffuse_texture: Texture) -> Self {
        Self {
            diffuse_texture: Some(diffuse_texture),
            ..self
        }
    }
    pub fn diffuse_color(self, diffuse_color: Color) -> Self {
        Self {
            diffuse_color: Some(diffuse_color),
            ..self
        }
    }
    pub fn normal_texture(self, normal_texture: Texture) -> Self {
        Self {
            normal_map: Some(normal_texture),
            ..self
        }
    }
    pub fn index_of_refraction(self, ior: f32) -> Self {
        Self {
            ior: Some(ior),
            ..self
        }
    }
    pub fn specular(self, specular: f32) -> Self {
        Self {
            specular: Some(specular),
            ..self
        }
    }
    pub fn roughness(self, roughness: f32) -> Self {
        Self {
            roughness: Some(roughness),
            ..self
        }
    }
    pub fn ambient(self, ambient: Color) -> Self {
        Self {
            ambient: Some(ambient),
            ..self
        }
    }
    pub fn build(self, device: &wgpu::Device, queue: &wgpu::Queue) -> PbrMaterial {
        let diffuse_color = UniformBuffer::new(self.diffuse_color.unwrap_or(Color::WHITE), device);
        let diffuse_texture = self.diffuse_texture.unwrap_or_else(|| {
            let default_white_image: DynamicImage =
                ImageBuffer::from_pixel(2, 2, image::Rgba([255_u8, 255_u8, 255_u8, 255_u8])).into();
            Texture::new(default_white_image, device, queue)
        });
        let ior = UniformBuffer::new(self.ior.unwrap_or(1.5), device);
        let specular = UniformBuffer::new(self.specular.unwrap_or(0.5), device);
        let roughness = UniformBuffer::new(self.roughness.unwrap_or(1.0), device);
        let normal_map = self.normal_map.unwrap_or_else(|| {
            let default_normal_map: DynamicImage =
                ImageBuffer::from_pixel(2, 2, image::Rgba([127_u8, 127_u8, 127_u8, 255_u8])).into();
            Texture::new(default_normal_map, device, queue)
        });
        let ambient = UniformBuffer::new(self.ambient.unwrap_or(Color::WHITE * 0.01), device);

        let bind_group = BindGroupBuilder::new()
            .texture(&diffuse_texture)
            .uniform(&diffuse_color.buffer)
            .texture(&normal_map)
            .uniform(&specular.buffer)
            .uniform(&ior.buffer)
            .uniform(&roughness.buffer)
            .uniform(&ambient.buffer)
            .build(device);
        PbrMaterial {
            diffuse_texture,
            diffuse_color,
            bind_group,
            normal_map,
            ior,
            specular,
            roughness,
            ambient,
        }
    }
}

impl<'a> Material<'a> for PbrMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn shader() -> ShaderModuleDescriptor<'a> {
        include_wgsl!("pbr.wgsl")
    }
}

impl<'a> Material<'a> for UnlitMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn shader() -> ShaderModuleDescriptor<'a> {
        include_wgsl!("unlit.wgsl")
    }
}
impl<'a> Material<'a> for LitMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
    fn shader() -> ShaderModuleDescriptor<'a> {
        include_wgsl!("lit.wgsl")
    }
}
pub struct MeshPipelineBuilder;

impl MeshPipelineBuilder {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a, M: Material<'a>>(
        material: &'a M,
        other_bind_group: &'a wgpu::BindGroupLayout,
    ) -> RenderPipelineBuilder<'a> {
        let shader = M::shader();
        let bind_group = material.bind_group();
        RenderPipelineBuilder::new(shader)
            .add_bind_group(other_bind_group)
            .add_bind_group(&bind_group.layout)
    }
}
