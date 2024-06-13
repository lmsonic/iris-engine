use egui::Slider;
use glam::Vec3;
use iris_engine::{
    geometry::shapes::Sphere,
    renderer::{
        bind_group::{BindGroup, BindGroupBuilder},
        buffer::{IndexBuffer, StorageBuffer, UniformBuffer, VertexBuffer},
        camera::{GpuCamera, OrbitCamera},
        color::Color,
        gui::{array3_edit, color_edit, direction_edit, drag_angle_clamp, float_edit, vec3_edit},
        light::{DirectionalLight, GpuLight, Light, PointLight, SpotLight},
        material::{LitMaterial, LitMaterialBuilder, MeshPipelineBuilder},
        mesh::{Meshable, Vertex},
        render_pipeline::{RenderPassBuilder, RenderPipelineWire},
        texture::Texture,
    },
};

struct Example {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer,
    bind_group: BindGroup,
    camera: OrbitCamera,
    camera_uniform: UniformBuffer<GpuCamera>,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
    material: LitMaterial,
    depth_texture: Texture,
    clear_color: Color,
    light_storage: StorageBuffer<Vec<GpuLight>>,
}

impl iris_engine::renderer::app::App for Example {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::POLYGON_MODE_LINE
    }

    fn gui(&mut self, ctx: &egui::Context, queue: &wgpu::Queue) {
        egui::Window::new("Sphere Lit example")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .show(ctx, |ui| {
                if color_edit(ui, &mut self.material.diffuse_color.data, "Diffuse Color") {
                    self.material.diffuse_color.update(queue);
                }
                if color_edit(ui, &mut self.material.specular_color.data, "Specular Color") {
                    self.material.specular_color.update(queue);
                }

                if float_edit(
                    ui,
                    &mut self.material.specular_exponent.data,
                    "Specular Exponent",
                    0.0..=1000.0,
                ) {
                    self.material.specular_exponent.update(queue);
                }
                let mut changed = false;
                let mut indices = vec![];
                for (i, gpu_light) in &mut self.light_storage.data.iter_mut().enumerate() {
                    match Light::from_gpu(gpu_light) {
                        Light::DirectionalLight(mut light) => {
                            ui.label("Directional Light");
                            changed |= color_edit(ui, &mut light.color, "Color");
                            changed |= direction_edit(ui, &mut light.direction, "Direction");
                            if changed {
                                *gpu_light = light.to_gpu();
                            }
                        }
                        Light::PointLight(mut light) => {
                            ui.label("Point Light");
                            changed |= color_edit(ui, &mut light.color, "Color");
                            changed |= vec3_edit(ui, &mut light.position, "Position", -10.0..=10.0);
                            changed |= float_edit(ui, &mut light.range, "Range", 1.0..=100.0);
                            changed |= array3_edit(
                                ui,
                                &mut light.attenuation,
                                "Attenuation function",
                                0.0..=5.0,
                            );

                            if changed {
                                *gpu_light = light.to_gpu();
                            }
                        }
                        Light::SpotLight(mut light) => {
                            ui.label("Spot Light");
                            changed |= color_edit(ui, &mut light.color, "Color");
                            changed |= direction_edit(ui, &mut light.direction, "Direction");
                            changed |= vec3_edit(ui, &mut light.position, "Position", -10.0..=10.0);
                            changed |= float_edit(ui, &mut light.range, "Range", 1.0..=100.0);
                            ui.horizontal(|ui| {
                                changed |=
                                    drag_angle_clamp(ui, &mut light.outer_cutoff, 0.0..=90.0)
                                        .changed();
                                light.outer_cutoff = f32::max(0.0, light.outer_cutoff);
                                ui.label("Cutoff");
                            });
                            if changed {
                                *gpu_light = light.to_gpu();
                            }
                        }
                    };
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
                        .push(DirectionalLight::default().to_gpu());
                    changed = true;
                }
                if ui.button("Add Point Light").clicked() {
                    self.light_storage.data.push(PointLight::default().to_gpu());
                    changed = true;
                }
                if ui.button("Add Spot Light").clicked() {
                    self.light_storage.data.push(SpotLight::default().to_gpu());
                    changed = true;
                }

                if changed || !indices.is_empty() {
                    self.light_storage.update_vec(queue);
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
        let cube = Sphere::new(1.0).mesh();
        let vertices = cube.vertices();
        let indices = cube.indices();
        let vertex_buffer = VertexBuffer::new(vertices, device);
        let index_buffer = IndexBuffer::new(indices, device);
        let aspect_ratio = config.width as f32 / config.height as f32;
        let camera = OrbitCamera::new(2.0, aspect_ratio);

        let camera_uniform = UniformBuffer::new(camera.to_gpu(), device);

        let directional_light = DirectionalLight::new(Color::WHITE, Vec3::NEG_ONE);

        let light_storage =
            StorageBuffer::from_container(vec![directional_light.to_gpu()], device, queue, 100);

        let bind_group = BindGroupBuilder::new()
            .uniform(&camera_uniform.buffer)
            .storage_buffer(&light_storage.buffer)
            .build(device);
        let texture = Texture::from_path("examples/bricks.jpg", device, queue);
        let normal = Texture::from_path("examples/bricks_normal.jpg", device, queue);
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
            camera,
            camera_uniform,
            pipeline,
            pipeline_wire: _pipeline_wire,
            material,
            depth_texture,
            clear_color,
            light_storage,
        }
    }

    fn input(&mut self, event: winit::event::WindowEvent, queue: &wgpu::Queue) {
        if self.camera.input(event) {
            self.camera_uniform.data = self.camera.to_gpu();
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
        self.camera.set_projection(aspect_ratio);
        self.camera_uniform.data = self.camera.to_gpu();
        self.camera_uniform.update(queue);
        self.depth_texture = Texture::depth(device, config.width, config.height)
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = RenderPassBuilder::new()
                .clear_color(self.clear_color.into())
                .depth(&self.depth_texture.view)
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
