use glam::Vec3;
use iris_engine::{
    geometry::shapes::Triangle,
    renderer::{
        app::{AppContext, SurfaceWrapper},
        bind_group::{BindGroup, BindGroupBuilder},
        buffer::{IndexBuffer, UniformBuffer, VertexBuffer},
        camera::OrbitCamera,
        color::Color,
        gui::color_edit,
        material::{Material, MaterialPipelineBuilder, UnlitMaterial, UnlitMaterialBuilder},
        mesh::{Meshable, Vertex},
        render_pipeline::{RenderPassBuilder, RenderPipelineWire},
    },
};

struct Example {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer,
    bind_group: BindGroup,
    camera_uniform: UniformBuffer<OrbitCamera>,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
    material: UnlitMaterial,
    clear_color: Color,
}

impl iris_engine::renderer::app::App for Example {
    fn gui(&mut self, ctx: &egui::Context, app: &AppContext, surface: &SurfaceWrapper) {
        egui::Window::new("Triangle example")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .show(ctx, |ui| {
                if self.material.gui(ui, &app.queue, &app.device) {
                    self.pipeline = MaterialPipelineBuilder::new(&self.material)
                        .add_bind_group(&self.bind_group.layout)
                        .build::<Vertex>(&app.device, surface.config.format);
                }

                color_edit(ui, &mut self.clear_color, "Clear Color");
            });
    }
    fn gui_register(
        &mut self,
        egui_renderer: &mut iris_engine::renderer::egui_renderer::EguiRenderer,
        device: &wgpu::Device,
    ) {
        self.material.gui_register(egui_renderer, device);
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let triangle = Triangle::new(Vec3::X, Vec3::NEG_X, Vec3::new(0.0, 1.0, 1.0)).mesh();
        let vertices = triangle.vertices();
        let indices = triangle.indices();
        let vertex_buffer = VertexBuffer::new(vertices, device);
        let index_buffer = IndexBuffer::new(indices, device);
        let aspect_ratio = config.width as f32 / config.height as f32;
        let camera = OrbitCamera::new(2.0, aspect_ratio);

        let camera_uniform = UniformBuffer::new(camera, device);

        let bind_group = BindGroupBuilder::new()
            .uniform(&camera_uniform.buffer)
            .build(device);

        let material = UnlitMaterialBuilder::new().build(device, queue);
        let pipeline = MaterialPipelineBuilder::new(&material)
            .add_bind_group(&bind_group.layout)
            .build::<Vertex>(device, config.format);

        let pipeline_wire = device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
            .then(|| {
                RenderPipelineWire::new()
                    .add_bind_group(&bind_group.layout)
                    .polygon_mode(wgpu::PolygonMode::Line)
                    .cull_mode(None)
                    .build::<Vertex>(device, config.format)
            });
        let clear_color = Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
        };

        // Done
        Self {
            vertex_buffer,
            index_buffer,
            bind_group,
            camera_uniform,
            pipeline,
            pipeline_wire,
            material,
            clear_color,
        }
    }

    fn input(&mut self, event: winit::event::WindowEvent, queue: &wgpu::Queue) {
        if self.camera_uniform.data.input(&event) {
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
        self.camera_uniform.data.set_projection(aspect_ratio);
        self.camera_uniform.update(queue);
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = RenderPassBuilder::new()
                .clear_color(self.clear_color.into())
                .build(&mut encoder, view);
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.material.bind_group.bind_group, &[]);
            rpass.set_bind_group(1, &self.bind_group.bind_group, &[]);
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
