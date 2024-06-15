use egui::Ui;
use glam::{Affine3A, Mat4};

use super::{
    bind_group::{BindGroup, BindGroupBuilder},
    buffer::UniformBuffer,
    egui_renderer::EguiRenderer,
    gui::{change_material, transform_edit},
    material::{Material, MaterialPipelineBuilder},
    mesh::Mesh,
    render_pipeline::RenderPipelineBuilder,
};

#[derive(Debug)]
pub struct Model {
    transform: Affine3A,
    mesh: Mesh,
    material: Box<dyn for<'a> Material<'a>>,
    transform_uniform: UniformBuffer<Mat4>,
    transform_bind_group: BindGroup,
}

impl Model {
    pub fn new<M: for<'a> Material<'a> + 'static>(
        transform: Affine3A,
        mesh: Mesh,
        material: M,
        device: &wgpu::Device,
    ) -> Self {
        let transform_uniform = UniformBuffer::new(Mat4::from(transform), device);
        let transform_bind_group = BindGroupBuilder::new()
            .uniform(&transform_uniform.buffer)
            .build(device);
        Self {
            transform,
            mesh,
            material: Box::new(material),
            transform_uniform,
            transform_bind_group,
        }
    }

    pub fn gui_register(&mut self, egui_renderer: &mut EguiRenderer, device: &wgpu::Device) {
        self.material.gui_register(egui_renderer, device);
    }
    pub fn gui(&mut self, ui: &mut Ui, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let mut changed = false;
        if transform_edit(ui, &mut self.transform) {
            self.set_transform(self.transform, queue);
        }
        ui.collapsing("Material", |ui| {
            changed |= self.material.gui(ui, queue, device);
            changed |= change_material(ui, &mut self.material, device, queue);
        });

        changed
    }

    pub fn pipeline(&self) -> RenderPipelineBuilder {
        MaterialPipelineBuilder::new(self.material.as_ref())
            .add_bind_group(&self.transform_bind_group.layout)
    }

    pub fn set_transform(&mut self, transform: Affine3A, queue: &wgpu::Queue) {
        self.transform = transform;
        self.transform_uniform.data = transform.into();
        self.transform_uniform.update(queue);
    }

    pub fn set_mesh(&mut self, mesh: Mesh) {
        self.mesh = mesh;
    }

    pub fn set_material<M: for<'a> Material<'a> + 'static>(&mut self, material: M) {
        self.material = Box::new(material);
    }

    pub const fn transform_bind_group(&self) -> &BindGroup {
        &self.transform_bind_group
    }

    pub const fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub const fn transform(&self) -> Affine3A {
        self.transform
    }

    pub fn material(&self) -> &dyn Material {
        self.material.as_ref()
    }
}
