use std::f32::consts;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use iris_engine::{
    geometry::shapes::{Cuboid, Sphere},
    renderer::{
        bind_group::BindGroup,
        buffer::{DataBuffer, IndexBuffer, VertexBuffer},
        camera::OrbitCamera,
        mesh::{Meshable, Vertex},
        render_pipeline::RenderPipelineBuilder,
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
    position: Vec3,
    _pad: f32,
}

impl CameraUniform {
    fn new(projection: Mat4, view: Mat4, position: Vec3) -> Self {
        Self {
            projection,
            view,
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
        _queue: &wgpu::Queue,
    ) -> Self {
        let triangle = Sphere::new(1.0).mesh();
        let vertices = triangle.vertices();
        let indices = triangle.indices();
        let vertex_buffer = VertexBuffer::new(vertices, device);
        let index_buffer = IndexBuffer::new(indices, device);
        let camera = OrbitCamera::new(2.0);

        let aspect_ratio = config.width as f32 / config.height as f32;
        let camera_uniform = DataBuffer::uniform(
            CameraUniform::new(
                Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0),
                camera.view(),
                camera.position(),
            ),
            device,
        );
        let bind_group = BindGroup::new(device, &[&camera_uniform.buffer], &[]);
        let shader = include_wgsl!("../basic_shader.wgsl");

        let pipeline = RenderPipelineBuilder::new(device, shader.clone(), config.format)
            .bind_group(&bind_group.layout)
            .fragment_entry("fs_main")
            .cull_mode(None)
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

        // Done
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
            self.camera_uniform.data.view = self.camera.view();
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
        self.camera_uniform.data.projection =
            Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
        self.camera_uniform.update(queue);
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
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
