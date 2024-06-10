use glam::Vec3;
use iris_engine::{
    geometry::shapes::Cuboid,
    renderer::{
        bind_group::{BindGroup, BindGroupBuilder},
        buffer::{IndexBuffer, UniformBuffer, VertexBuffer},
        camera::{GpuCamera, OrbitCamera},
        color::Color,
        material::{MeshPipelineBuilder, UnlitMaterial, UnlitMaterialBuilder},
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
    material: UnlitMaterial,
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
        let cube = Cuboid::new(Vec3::splat(1.0)).mesh();
        let vertices = cube.vertices();
        let indices = cube.indices();
        let vertex_buffer = VertexBuffer::new(vertices, device);
        let index_buffer = IndexBuffer::new(indices, device);
        let aspect_ratio = config.width as f32 / config.height as f32;
        let camera = OrbitCamera::new(2.0, aspect_ratio);

        let camera_uniform = UniformBuffer::new(camera.to_gpu(), device);

        let texture = Texture::from_path("checkerboard.png", device, queue);

        let bind_group = BindGroupBuilder::new()
            .uniform(&camera_uniform.buffer)
            .build(device);

        let material = UnlitMaterialBuilder::new()
            .diffuse_texture(texture)
            .build(device, queue);
        let pipeline = MeshPipelineBuilder::new(&material, &bind_group.layout)
            .build::<Vertex>(device, config.format);

        let pipeline_wire = if device
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
            let mut rpass = RenderPassBuilder::new()
                .clear_color(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                })
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
