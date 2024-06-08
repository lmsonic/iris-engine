use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use iris_engine::{
    geometry::shapes::Sphere,
    renderer::{
        bind_group::BindGroup,
        buffer::{DataBuffer, IndexBuffer, VertexBuffer},
        camera::OrbitCamera,
        color::Color,
        light::{DirectionalLight, PointLight, SpotLight},
        mesh::{Meshable, Vertex},
        render_pipeline::{RenderPassBuilder, RenderPipelineBuilder},
        texture::Texture,
    },
};

use wgpu::include_wgsl;

struct Example {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer,
    bind_group: BindGroup,
    camera: OrbitCamera,
    camera_uniform: DataBuffer<CameraUniform>,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    projection: Mat4,
    view: Mat4,
    inverse_view: Mat4,
    position: Vec3,
    _pad: f32,
}

impl CameraUniform {
    fn set_view(&mut self, view: Mat4) {
        self.view = view;
        self.inverse_view = view.inverse().transpose();
    }
}

impl CameraUniform {
    fn new(projection: Mat4, view: Mat4, position: Vec3) -> Self {
        Self {
            projection,
            view,
            inverse_view: view.inverse().transpose(),
            position,
            _pad: 0.0,
        }
    }
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
        let sphere = Sphere::new(1.0).mesh();
        let vertices = sphere.vertices();
        let indices = sphere.indices();
        let vertex_buffer = VertexBuffer::new(vertices, device);
        let index_buffer = IndexBuffer::new(indices, device);
        let camera = OrbitCamera::new(2.0);

        let aspect_ratio = config.width as f32 / config.height as f32;

        let camera_uniform = DataBuffer::uniform(
            CameraUniform::new(
                camera.projection(aspect_ratio),
                camera.view(),
                camera.position(),
            ),
            device,
        );

        let directional_light = DirectionalLight::new(Color::RED, Vec3::ONE);
        let point_light = PointLight::new(Color::GREEN, Vec3::ONE);
        let spot_light = SpotLight::new(
            Color::BLUE,
            Vec3::Y * 2.0,
            Vec3::NEG_Y,
            100.0,
            f32::to_radians(45.0),
        );
        let directional_light_uniform = DataBuffer::uniform(directional_light.to_gpu(), device);
        let point_light_uniform = DataBuffer::uniform(point_light.to_gpu(), device);
        let spot_light_uniform = DataBuffer::uniform(spot_light.to_gpu(), device);
        let texture = Texture::new("checkerboard.png", device, queue);
        let bind_group = BindGroup::new(
            device,
            &[
                &camera_uniform.buffer,
                &directional_light_uniform.buffer,
                &point_light_uniform.buffer,
                &spot_light_uniform.buffer,
            ],
            &[&texture],
        );
        let shader = include_wgsl!("../lit.wgsl");

        let pipeline = RenderPipelineBuilder::new(device, shader.clone(), config.format)
            .bind_group(&bind_group.layout)
            // .cull_mode(None)
            .build::<Vertex>();

        let pipeline_wire = if device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            Some(
                RenderPipelineBuilder::new(device, shader, config.format)
                    .bind_group(&bind_group.layout)
                    .fragment_entry("fs_wire")
                    .polygon_mode(wgpu::PolygonMode::Line)
                    .cull_mode(None)
                    .build::<Vertex>(),
            )
        } else {
            None
        };
        let pipeline_wire = None;

        Example {
            vertex_buffer,
            index_buffer,
            bind_group,
            camera,
            camera_uniform,
            pipeline,
            pipeline_wire,
        }
    }

    fn input(&mut self, event: winit::event::WindowEvent, queue: &wgpu::Queue) {
        if self.camera.input(event) {
            self.camera_uniform.data.position = self.camera.position();
            self.camera_uniform.data.set_view(self.camera.view());
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
        self.camera_uniform.data.projection = self.camera.projection(aspect_ratio);
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
