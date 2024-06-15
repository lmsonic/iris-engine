use std::fmt::Debug;

use egui::Ui;
use image::{DynamicImage, ImageBuffer};
use wgpu::{include_wgsl, ShaderModuleDescriptor};

use super::{
    bind_group::{BindGroup, BindGroupBuilder},
    buffer::UniformBuffer,
    color::Color,
    egui_renderer::EguiRenderer,
    gui::{color_edit, float_edit, texture_edit},
    render_pipeline::RenderPipelineBuilder,
    texture::Texture,
};

pub trait Material<'a>: Debug {
    fn shader(&self) -> ShaderModuleDescriptor<'a>;
    fn bind_group(&self) -> &BindGroup;
    fn gui(&mut self, _ui: &mut Ui, _queue: &wgpu::Queue, _device: &wgpu::Device) -> bool {
        false
    }
    fn gui_register(&mut self, _egui_renderer: &mut EguiRenderer, _device: &wgpu::Device) {}
}
#[derive(Debug)]
pub struct UnlitMaterial {
    pub diffuse_texture: Texture,
    pub diffuse_color: UniformBuffer<Color>,
    pub bind_group: BindGroup,
}

impl UnlitMaterial {
    const DEFAULT_DIFFUSE_COLOR: Color = Color::WHITE;
    pub fn default_diffuse_texture() -> DynamicImage {
        ImageBuffer::from_pixel(1, 1, image::Rgba([255_u8, 255_u8, 255_u8, 255_u8])).into()
    }
    pub fn from_pbr(value: PbrMaterial, device: &wgpu::Device) -> Self {
        let mut s = Self {
            diffuse_texture: value.diffuse_texture,
            diffuse_color: value.diffuse_color,
            bind_group: value.bind_group,
        };
        s.rebuild_bind_group(device);
        s
    }
    pub fn from_lit(value: LitMaterial, device: &wgpu::Device) -> Self {
        let mut s = Self {
            bind_group: value.bind_group,
            diffuse_texture: value.diffuse_texture,
            diffuse_color: value.diffuse_color,
        };
        s.rebuild_bind_group(device);
        s
    }
    fn rebuild_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group = BindGroupBuilder::new()
            .texture(&self.diffuse_texture)
            .uniform(&self.diffuse_color.buffer)
            .build(device);
    }
}

impl<'a> Material<'a> for UnlitMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn shader(&self) -> ShaderModuleDescriptor<'a> {
        include_wgsl!("shaders/unlit.wgsl")
    }
    fn gui(&mut self, ui: &mut Ui, queue: &wgpu::Queue, device: &wgpu::Device) -> bool {
        if color_edit(ui, &mut self.diffuse_color.data, "Diffuse Color") {
            self.diffuse_color.update(queue);
        }
        if let Some(id) = self.diffuse_texture.egui_id {
            if texture_edit(ui, id, "Diffuse Texture") {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Ok(texture) = Texture::from_path(path, device, queue) {
                        self.diffuse_texture = texture;
                        self.rebuild_bind_group(device);
                        return true;
                    }
                }
            }
        }
        false
    }
    fn gui_register(&mut self, egui_renderer: &mut EguiRenderer, device: &wgpu::Device) {
        egui_renderer.register_texture(&mut self.diffuse_texture, device);
    }
}

#[derive(Default, Debug)]
pub struct UnlitMaterialBuilder {
    diffuse_texture: Option<Texture>,
    diffuse_color: Option<Color>,
}

impl UnlitMaterialBuilder {
    pub fn new() -> Self {
        Self::default()
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
        let diffuse_color = UniformBuffer::new(
            self.diffuse_color
                .unwrap_or(UnlitMaterial::DEFAULT_DIFFUSE_COLOR),
            device,
        );
        let diffuse_texture = self.diffuse_texture.unwrap_or_else(|| {
            Texture::new(UnlitMaterial::default_diffuse_texture(), device, queue)
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

#[derive(Debug)]
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
    const DEFAULT_DIFFUSE_COLOR: Color = Color::WHITE;
    pub fn default_diffuse_texture() -> DynamicImage {
        ImageBuffer::from_pixel(1, 1, image::Rgba::<u8>([255, 255, 255, 255])).into()
    }
    const DEFAULT_SPECULAR_COLOR: Color = Color::WHITE;
    pub fn default_normal_texture() -> DynamicImage {
        ImageBuffer::from_pixel(1, 1, image::Rgba::<u8>([0, 0, 255, 255])).into()
    }
    const DEFAULT_SPECULAR_EXPONENT: f32 = 100.0;
    const DEFAULT_AMBIENT_COLOR: Color = Color::new(0.01, 0.01, 0.01);

    pub fn from_pbr(value: PbrMaterial, device: &wgpu::Device) -> Self {
        let mut s = Self {
            bind_group: value.bind_group,
            diffuse_texture: value.diffuse_texture,
            diffuse_color: value.diffuse_color,
            normal_map: value.normal_map,
            specular_color: UniformBuffer::new(Self::DEFAULT_SPECULAR_COLOR, device),
            specular_exponent: UniformBuffer::new(Self::DEFAULT_SPECULAR_EXPONENT, device),
            ambient: value.ambient,
        };
        s.rebuild_bind_group(device);
        s
    }
    pub fn from_unlit(value: UnlitMaterial, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut s = Self {
            bind_group: value.bind_group,
            diffuse_texture: value.diffuse_texture,
            diffuse_color: value.diffuse_color,
            normal_map: Texture::new(Self::default_normal_texture(), device, queue),
            specular_color: UniformBuffer::new(Self::DEFAULT_SPECULAR_COLOR, device),
            specular_exponent: UniformBuffer::new(Self::DEFAULT_SPECULAR_EXPONENT, device),
            ambient: UniformBuffer::new(Self::DEFAULT_AMBIENT_COLOR, device),
        };
        s.rebuild_bind_group(device);
        s
    }
    fn rebuild_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group = BindGroupBuilder::new()
            .texture(&self.diffuse_texture)
            .uniform(&self.diffuse_color.buffer)
            .texture(&self.normal_map)
            .uniform(&self.specular_color.buffer)
            .uniform(&self.specular_exponent.buffer)
            .uniform(&self.ambient.buffer)
            .build(device);
    }
}

impl<'a> Material<'a> for LitMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
    fn shader(&self) -> ShaderModuleDescriptor<'a> {
        include_wgsl!("shaders/lit.wgsl")
    }
    fn gui(&mut self, ui: &mut Ui, queue: &wgpu::Queue, device: &wgpu::Device) -> bool {
        if color_edit(ui, &mut self.diffuse_color.data, "Diffuse Color") {
            self.diffuse_color.update(queue);
        }
        if let Some(id) = self.diffuse_texture.egui_id {
            if texture_edit(ui, id, "Diffuse Texture") {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Ok(texture) = Texture::from_path(path, device, queue) {
                        self.diffuse_texture = texture;
                        self.rebuild_bind_group(device);
                        return true;
                    }
                }
            }
        }
        if let Some(id) = self.normal_map.egui_id {
            if texture_edit(ui, id, "Normal Texture") {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Ok(texture) = Texture::from_path(path, device, queue) {
                        self.normal_map = texture;
                        self.rebuild_bind_group(device);
                        return true;
                    }
                }
            }
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
        false
    }
    fn gui_register(&mut self, egui_renderer: &mut EguiRenderer, device: &wgpu::Device) {
        egui_renderer.register_texture(&mut self.diffuse_texture, device);
        egui_renderer.register_texture(&mut self.normal_map, device);
    }
}

#[derive(Default, Debug)]
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
        Self::default()
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
        let diffuse_color = UniformBuffer::new(
            self.diffuse_color
                .unwrap_or(LitMaterial::DEFAULT_DIFFUSE_COLOR),
            device,
        );
        let diffuse_texture = self
            .diffuse_texture
            .unwrap_or_else(|| Texture::new(LitMaterial::default_diffuse_texture(), device, queue));
        let specular_color = UniformBuffer::new(
            self.specular_color
                .unwrap_or(LitMaterial::DEFAULT_SPECULAR_COLOR),
            device,
        );

        let specular_exponent = UniformBuffer::new(
            self.specular_exponent
                .unwrap_or(LitMaterial::DEFAULT_SPECULAR_EXPONENT),
            device,
        );
        let normal_map = self
            .normal_map
            .unwrap_or_else(|| Texture::new(LitMaterial::default_normal_texture(), device, queue));
        let ambient = UniformBuffer::new(
            self.ambient.unwrap_or(LitMaterial::DEFAULT_AMBIENT_COLOR),
            device,
        );
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
            normal_map,
            specular_color,
            specular_exponent,
            ambient,
            bind_group,
        }
    }
}

#[derive(Debug)]
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
    const DEFAULT_DIFFUSE_COLOR: Color = Color::WHITE;
    pub fn default_diffuse_texture() -> DynamicImage {
        ImageBuffer::from_pixel(1, 1, image::Rgba::<u8>([255, 255, 255, 255])).into()
    }
    pub fn default_normal_texture() -> DynamicImage {
        ImageBuffer::from_pixel(1, 1, image::Rgba::<u8>([0, 0, 255, 255])).into()
    }
    const DEFAULT_SPECULAR: f32 = 0.5;
    const DEFAULT_IOR: f32 = 1.4;
    const DEFAULT_ROUGHNESS: f32 = 1.0;
    const DEFAULT_AMBIENT_COLOR: Color = Color::new(0.01, 0.01, 0.01);

    pub fn from_lit(value: LitMaterial, device: &wgpu::Device) -> Self {
        let mut s = Self {
            bind_group: value.bind_group,
            diffuse_texture: value.diffuse_texture,
            diffuse_color: value.diffuse_color,
            normal_map: value.normal_map,
            ambient: value.ambient,
            specular: UniformBuffer::new(Self::DEFAULT_SPECULAR, device),
            ior: UniformBuffer::new(Self::DEFAULT_IOR, device),
            roughness: UniformBuffer::new(Self::DEFAULT_ROUGHNESS, device),
        };
        s.rebuild_bind_group(device);
        s
    }
    pub fn from_unlit(value: UnlitMaterial, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut s = Self {
            bind_group: value.bind_group,
            diffuse_texture: value.diffuse_texture,
            diffuse_color: value.diffuse_color,
            normal_map: Texture::new(Self::default_normal_texture(), device, queue),
            ambient: UniformBuffer::new(Self::DEFAULT_AMBIENT_COLOR, device),
            specular: UniformBuffer::new(Self::DEFAULT_SPECULAR, device),
            ior: UniformBuffer::new(Self::DEFAULT_IOR, device),
            roughness: UniformBuffer::new(Self::DEFAULT_ROUGHNESS, device),
        };
        s.rebuild_bind_group(device);
        s
    }
    fn rebuild_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group = BindGroupBuilder::new()
            .texture(&self.diffuse_texture)
            .uniform(&self.diffuse_color.buffer)
            .texture(&self.normal_map)
            .uniform(&self.specular.buffer)
            .uniform(&self.ior.buffer)
            .uniform(&self.roughness.buffer)
            .uniform(&self.ambient.buffer)
            .build(device);
    }
}

impl<'a> Material<'a> for PbrMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn shader(&self) -> ShaderModuleDescriptor<'a> {
        include_wgsl!("shaders/pbr.wgsl")
    }
    fn gui(&mut self, ui: &mut Ui, queue: &wgpu::Queue, device: &wgpu::Device) -> bool {
        if color_edit(ui, &mut self.diffuse_color.data, "Diffuse Color") {
            self.diffuse_color.update(queue);
        }
        if let Some(id) = self.diffuse_texture.egui_id {
            if texture_edit(ui, id, "Diffuse Texture") {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Ok(texture) = Texture::from_path(path, device, queue) {
                        self.diffuse_texture = texture;
                        self.rebuild_bind_group(device);
                        return true;
                    }
                }
            }
        }
        if let Some(id) = self.normal_map.egui_id {
            if texture_edit(ui, id, "Normal Texture") {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Ok(texture) = Texture::from_path(path, device, queue) {
                        self.normal_map = texture;
                        self.rebuild_bind_group(device);
                        return true;
                    }
                }
            }
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
        false
    }
    fn gui_register(&mut self, egui_renderer: &mut EguiRenderer, device: &wgpu::Device) {
        egui_renderer.register_texture(&mut self.diffuse_texture, device);
        egui_renderer.register_texture(&mut self.normal_map, device);
    }
}

#[derive(Default, Debug)]
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
        Self::default()
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
        let diffuse_color = UniformBuffer::new(
            self.diffuse_color
                .unwrap_or(PbrMaterial::DEFAULT_DIFFUSE_COLOR),
            device,
        );
        let diffuse_texture = self
            .diffuse_texture
            .unwrap_or_else(|| Texture::new(PbrMaterial::default_diffuse_texture(), device, queue));
        let ior = UniformBuffer::new(self.ior.unwrap_or(PbrMaterial::DEFAULT_IOR), device);
        let specular = UniformBuffer::new(
            self.specular.unwrap_or(PbrMaterial::DEFAULT_SPECULAR),
            device,
        );
        let roughness = UniformBuffer::new(
            self.roughness.unwrap_or(PbrMaterial::DEFAULT_ROUGHNESS),
            device,
        );
        let normal_map = self
            .normal_map
            .unwrap_or_else(|| Texture::new(PbrMaterial::default_normal_texture(), device, queue));
        let ambient = UniformBuffer::new(
            self.ambient.unwrap_or(PbrMaterial::DEFAULT_AMBIENT_COLOR),
            device,
        );

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
            normal_map,
            specular,
            ior,
            roughness,
            ambient,
            bind_group,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialPipelineBuilder;

impl MaterialPipelineBuilder {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a>(material: &'a dyn Material<'a>) -> RenderPipelineBuilder<'a> {
        let shader = material.shader();
        let bind_group = material.bind_group();
        RenderPipelineBuilder::new(shader).add_bind_group(&bind_group.layout)
    }
}
