use glam::Vec3;
use iris_engine::renderer::{
    bind_group::{BindGroup, BindGroupBuilder},
    buffer::{IndexBuffer, StorageBuffer, UniformBuffer, VertexBuffer},
    camera::{GpuCamera, OrbitCamera},
    color::Color,
    light::{DirectionalLight, PointLight, SpotLight},
    material::{LitMaterial, LitMaterialBuilder, MeshPipelineBuilder},
    mesh::{Mesh, Vertex},
    render_pipeline::{RenderPassBuilder, RenderPipelineWire},
    texture::Texture,
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
}

impl iris_engine::renderer::app::App for Example {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::POLYGON_MODE_LINE
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let boat = Mesh::from_obj("examples/plane/plane.obj");
        let vertices = boat.vertices();
        let indices = boat.indices();
        let vertex_buffer = VertexBuffer::new(vertices, device);
        let index_buffer = IndexBuffer::new(indices, device);
        let aspect_ratio = config.width as f32 / config.height as f32;
        let camera = OrbitCamera::new(2.0, aspect_ratio);

        let camera_uniform = UniformBuffer::new(camera.to_gpu(), device);

        let directional_light = DirectionalLight::new(Color::RED, Vec3::ONE);
        let point_light = PointLight::new(Color::WHITE, Vec3::ONE);
        let spot_light = SpotLight::new(
            Color::BLUE,
            Vec3::Y * 2.0,
            Vec3::NEG_Y,
            100.0,
            f32::to_radians(45.0),
        );
        let light_storage = StorageBuffer::new(
            [
                // directional_light.to_gpu(),
                point_light.to_gpu(),
                // spot_light.to_gpu(),
            ],
            device,
        );

        let bind_group = BindGroupBuilder::new()
            .uniform(&camera_uniform.buffer)
            .storage_buffer(&light_storage.buffer)
            .build(device);
        let texture = Texture::from_path("examples/plane/diffuse.jpg", device, queue);
        let normal = Texture::from_path("examples/plane/normal.png", device, queue);
        let material = LitMaterialBuilder::new()
            .diffuse_texture(texture)
            .normal_texture(normal)
            .build(device, queue);
        let pipeline =
            MeshPipelineBuilder::new(device, config.format, &material, &bind_group.layout)
                .build::<Vertex>();

        let mut pipeline_wire = if device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            Some(
                RenderPipelineWire::new(device, config.format)
                    .add_bind_group(&bind_group.layout)
                    .polygon_mode(wgpu::PolygonMode::Line)
                    .cull_mode(None)
                    .build::<Vertex>(),
            )
        } else {
            None
        };
        pipeline_wire = None;

        // Done
        Example {
            vertex_buffer,
            index_buffer,
            bind_group,
            camera,
            camera_uniform,
            pipeline,
            pipeline_wire,
            material,
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
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let aspect_ratio = config.width as f32 / config.height as f32;
        self.camera.set_projection(aspect_ratio);
        self.camera_uniform.data = self.camera.to_gpu();
        self.camera_uniform.update(queue);
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = RenderPassBuilder::new(&mut encoder, view)
                .clear_color(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                })
                .build();
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
