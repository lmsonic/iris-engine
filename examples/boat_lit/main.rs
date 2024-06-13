use glam::Vec3;
use iris_engine::renderer::{
    bind_group::{BindGroup, BindGroupBuilder},
    buffer::{IndexBuffer, StorageBufferVec, UniformBuffer, VertexBuffer},
    camera::OrbitCamera,
    color::Color,
    gui::color_edit,
    light::{DirectionalLight, Light, PointLight, SpotLight},
    material::{LitMaterial, LitMaterialBuilder, MeshPipelineBuilder},
    mesh::{Mesh, Vertex},
    render_pipeline::{RenderPassBuilder, RenderPipelineWire},
    texture::Texture,
};

struct Example {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer,
    bind_group: BindGroup,
    camera_uniform: UniformBuffer<OrbitCamera>,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
    material: LitMaterial,
    depth_texture: Texture,
    light_storage: StorageBufferVec<Light>,
    clear_color: Color,
}

impl iris_engine::renderer::app::App for Example {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::POLYGON_MODE_LINE
    }
    fn gui(&mut self, ctx: &egui::Context, queue: &wgpu::Queue) {
        egui::Window::new("Boat Lit example")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .show(ctx, |ui| {
                self.material.gui(ui, queue);
                let mut changed = false;
                let mut indices = vec![];
                for (i, gpu_light) in &mut self.light_storage.data.iter_mut().enumerate() {
                    changed |= gpu_light.gui(ui);
                    if ui.button("Remove Light").clicked() {
                        changed = true;
                        indices.push(i);
                    }
                }
                for i in indices.iter().rev() {
                    // Remove in reverse order
                    self.light_storage.data.remove(*i);
                }
                if ui.button("Add Directional Light").clicked() {
                    self.light_storage
                        .data
                        .push(DirectionalLight::default().into());
                    changed = true;
                }
                if ui.button("Add Point Light").clicked() {
                    self.light_storage.data.push(PointLight::default().into());
                    changed = true;
                }
                if ui.button("Add Spot Light").clicked() {
                    self.light_storage.data.push(SpotLight::default().into());
                    changed = true;
                }

                if changed || !indices.is_empty() {
                    self.light_storage.update(queue);
                }

                color_edit(ui, &mut self.clear_color, "Clear Color");
            });
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let boat = Mesh::from_obj("examples/boat/boat.obj");
        let vertices = boat.vertices();
        let indices = boat.indices();
        let vertex_buffer = VertexBuffer::new(vertices, device);
        let index_buffer = IndexBuffer::new(indices, device);
        let aspect_ratio = config.width as f32 / config.height as f32;
        let camera = OrbitCamera::new(2.0, aspect_ratio);

        let camera_uniform = UniformBuffer::new(camera, device);

        let point_light = PointLight {
            position: Vec3::ONE,
            ..Default::default()
        };
        let light_storage = StorageBufferVec::new(&[point_light.into()], device, queue, 16);

        let bind_group = BindGroupBuilder::new()
            .uniform(&camera_uniform.buffer)
            .storage_buffer(&light_storage.buffer)
            .build(device);
        let texture = Texture::from_path("examples/boat/boat.jpg", device, queue);
        let normal = Texture::from_path("examples/boat/boat.png", device, queue);
        let material = LitMaterialBuilder::new()
            .diffuse_texture(texture)
            .normal_texture(normal)
            .build(device, queue);
        let depth_texture = Texture::depth(device, config.width, config.height);
        let pipeline = MeshPipelineBuilder::new(&material, &bind_group.layout)
            .depth(depth_texture.texture.format())
            .build::<Vertex>(device, config.format);

        let mut _pipeline_wire = if device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            Some(
                RenderPipelineWire::new()
                    .add_bind_group(&bind_group.layout)
                    .polygon_mode(wgpu::PolygonMode::Line)
                    .cull_mode(None)
                    .build::<Vertex>(device, config.format),
            )
        } else {
            None
        };
        _pipeline_wire = None;

        let clear_color = Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
        };
        // Done
        Example {
            vertex_buffer,
            index_buffer,
            bind_group,
            camera_uniform,
            pipeline,
            pipeline_wire: _pipeline_wire,
            material,
            depth_texture,
            light_storage,
            clear_color,
        }
    }

    fn input(&mut self, event: winit::event::WindowEvent, queue: &wgpu::Queue) {
        if self.camera_uniform.data.input(event) {
            self.camera_uniform.update(queue);
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let aspect_ratio = config.width as f32 / config.height as f32;
        self.camera_uniform.data.set_projection(aspect_ratio);
        self.camera_uniform.update(queue);
        self.depth_texture = Texture::depth(device, config.width, config.height);
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = RenderPassBuilder::new()
                .depth(&self.depth_texture.view)
                .clear_color(self.clear_color.into())
                .build(&mut encoder, view);
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group.bind_group, &[]);
            rpass.set_bind_group(1, &self.material.bind_group.bind_group, &[]);
            rpass.set_index_buffer(
                self.index_buffer.buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
            rpass.draw_indexed(0..self.index_buffer.indices.len() as u32, 0, 0..1);
            if let Some(ref pipe) = self.pipeline_wire {
                rpass.set_pipeline(pipe);
                rpass.draw_indexed(0..self.index_buffer.indices.len() as u32, 0, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}

pub fn main() -> Result<(), winit::error::EventLoopError> {
    iris_engine::renderer::app::run::<Example>()
}
