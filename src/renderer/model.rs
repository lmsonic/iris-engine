use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use egui::Ui;
use glam::{Affine3A, Mat4, Vec4};

use crate::visibility::bounding_volume::{Aabb, Obb};

use super::{
    bind_group::{BindGroup, BindGroupBuilder},
    buffer::UniformBuffer,
    egui_renderer::EguiRenderer,
    gui::{change_material, transform_edit},
    material::{Material, MaterialPipelineBuilder},
    mesh::Mesh,
    render_pipeline::RenderPipelineBuilder,
    resources::VertexAttributeLayout,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Instance {
    pub x_axis: Vec4,
    pub y_axis: Vec4,
    pub z_axis: Vec4,
    pub w_axis: Vec4,
}

impl VertexAttributeLayout for Instance {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
            wgpu::vertex_attr_array![5=>Float32x4,6=>Float32x4,7=>Float32x4,8=>Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

impl Instance {
    pub const fn new(transform: Mat4) -> Self {
        Self {
            x_axis: transform.x_axis,
            y_axis: transform.y_axis,
            z_axis: transform.z_axis,
            w_axis: transform.w_axis,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InstancedModel {
    pub transform: Affine3A,
    pub mesh: Rc<Mesh>,
    pub bounding_box: Aabb,
}

impl InstancedModel {
    pub fn new(transform: Affine3A, mesh: Rc<Mesh>) -> Self {
        let bounding_box = mesh.calculate_bounding_box();
        Self {
            transform,
            mesh,
            bounding_box,
        }
    }
    pub fn bounding_box(&self) -> Obb {
        self.transform * self.bounding_box
    }
}

#[derive(Debug)]
pub struct Model {
    transform: Affine3A,
    mesh: Rc<Mesh>,
    material: Box<dyn for<'a> Material<'a>>,
    transform_uniform: UniformBuffer<Mat4>,
    transform_bind_group: BindGroup,
    bounding_box: Aabb,
}

impl Model {
    pub fn new<M: for<'a> Material<'a> + 'static>(
        transform: Affine3A,
        mesh: Rc<Mesh>,
        material: M,
        device: &wgpu::Device,
    ) -> Self {
        let transform_uniform = UniformBuffer::new(Mat4::from(transform), device);
        let transform_bind_group = BindGroupBuilder::new()
            .uniform(&transform_uniform.buffer)
            .build(device);
        let bounding_box = mesh.calculate_bounding_box();
        Self {
            transform,
            mesh,
            material: Box::new(material),
            transform_uniform,
            transform_bind_group,
            bounding_box,
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

    pub fn set_mesh(&mut self, mesh: Rc<Mesh>) {
        self.mesh = mesh;
        self.bounding_box = self.mesh.calculate_bounding_box();
    }

    pub fn set_material<M: for<'a> Material<'a> + 'static>(&mut self, material: M) {
        self.material = Box::new(material);
    }

    pub const fn transform_bind_group(&self) -> &BindGroup {
        &self.transform_bind_group
    }

    pub const fn mesh(&self) -> &Rc<Mesh> {
        &self.mesh
    }

    pub const fn transform(&self) -> Affine3A {
        self.transform
    }

    pub fn material(&self) -> &dyn Material {
        self.material.as_ref()
    }

    pub fn bounding_box(&self) -> Obb {
        self.transform * self.bounding_box
    }
}
