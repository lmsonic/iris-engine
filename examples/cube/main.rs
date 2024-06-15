use glam::Vec3;
use iris_engine::{
    geometry::shapes::Cuboid,
    renderer::{
        bind_group::{BindGroup, BindGroupBuilder},
        buffer::{IndexBuffer, StorageBufferArray, UniformBuffer, VertexBuffer},
        camera::OrbitCamera,
        color::Color,
        gui::{change_material, color_edit, lights_gui},
        light::{DirectionalLight, Light},
        material::{Material, MaterialPipelineBuilder, UnlitMaterialBuilder},
        mesh::{Meshable, Vertex},
        render_pipeline::{RenderPassBuilder, RenderPipelineWire},
        texture::Texture,
        wgpu_renderer::Renderer,
    },
};

struct Example {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer,
    bind_group: BindGroup,
    camera_uniform: UniformBuffer<OrbitCamera>,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
    material: Box<dyn for<'a> Material<'a>>,
    depth_texture: Texture,
    light_storage: StorageBufferArray<Light>,
    clear_color: Color,
}

impl iris_engine::renderer::app::App for Example {
    fn gui(&mut self, gui: &egui::Context, r: &Renderer) {
        egui::Window::new("Cube example")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .show(gui, |ui| {
                if self.material.gui(ui, &r.queue, &r.device) {
                    self.pipeline = MaterialPipelineBuilder::new(self.material.as_ref())
                        .add_bind_group(&self.bind_group.layout)
                        .depth(self.depth_texture.texture.format())
                        .build::<Vertex>(&r.device, r.config.format);
                }
                if change_material(ui, &mut self.material, &r.device, &r.queue) {
                    self.pipeline = MaterialPipelineBuilder::new(self.material.as_ref())
                        .add_bind_group(&self.bind_group.layout)
                        .depth(self.depth_texture.texture.format())
                        .build::<Vertex>(&r.device, r.config.format);
                }
                if lights_gui(ui, &mut self.light_storage.data) {
                    self.light_storage.update(&r.queue);
                }

                color_edit(ui, &mut self.clear_color, "Clear Color");
            });
    }
    fn gui_register(
        &mut self,
        egui_renderer: &mut iris_engine::renderer::egui_renderer::EguiRenderer,
        r: &mut Renderer,
    ) {
        self.material.gui_register(egui_renderer, &r.device);
    }

    fn init(r: &mut Renderer) -> Self {
        let cube = Cuboid::new(Vec3::splat(1.0)).mesh();
        let vertices = cube.vertices();
        let indices = cube.indices();
        let vertex_buffer = VertexBuffer::new(vertices, &r.device);
        let index_buffer = IndexBuffer::new(indices, &r.device);
        let aspect_ratio = r.config.width as f32 / r.config.height as f32;
        let camera = OrbitCamera::new(2.0, aspect_ratio);

        let camera_uniform = UniformBuffer::new(camera, &r.device);

        let directional_light = DirectionalLight {
            direction: Vec3::NEG_ONE,
            ..Default::default()
        };
        let light_storage =
            StorageBufferArray::new(&[directional_light.into()], &r.device, &r.queue, 16);

        let bind_group = BindGroupBuilder::new()
            .uniform(&camera_uniform.buffer)
            .storage_buffer(&light_storage.buffer)
            .build(&r.device);
        let texture = Texture::from_path("examples/checkerboard.png", &r.device, &r.queue).unwrap();
        let material = UnlitMaterialBuilder::new()
            .diffuse_texture(texture)
            .build(&r.device, &r.queue);
        let depth_texture = Texture::depth(&r.device, r.config.width, r.config.height);
        let pipeline = MaterialPipelineBuilder::new(&material)
            .add_bind_group(&bind_group.layout)
            .depth(depth_texture.texture.format())
            .build::<Vertex>(&r.device, r.config.format);

        let pipeline_wire = r
            .device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
            .then(|| {
                RenderPipelineWire::new()
                    .add_bind_group(&bind_group.layout)
                    .polygon_mode(wgpu::PolygonMode::Line)
                    .depth(depth_texture.texture.format())
                    .cull_mode(None)
                    .build::<Vertex>(&r.device, r.config.format)
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
            material: Box::new(material),
            depth_texture,
            light_storage,
            clear_color,
        }
    }

    fn input(&mut self, event: winit::event::WindowEvent, r: &mut Renderer) {
        if self.camera_uniform.data.input(&event) {
            self.camera_uniform.update(&r.queue);
        }
    }

    fn resize(&mut self, r: &mut Renderer) {
        let aspect_ratio = r.config.width as f32 / r.config.height as f32;
        self.camera_uniform.data.set_projection(aspect_ratio);
        self.camera_uniform.update(&r.queue);
        self.depth_texture = Texture::depth(&r.device, r.config.width, r.config.height);
    }

    fn render(&mut self, view: &wgpu::TextureView, r: &mut Renderer) {
        let mut encoder = r
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = RenderPassBuilder::new()
                .depth(&self.depth_texture.view)
                .clear_color(self.clear_color.into())
                .build(&mut encoder, view);
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.material.bind_group().bind_group, &[]);
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

        r.queue.submit(Some(encoder.finish()));
    }
}

pub fn main() -> Result<(), winit::error::EventLoopError> {
    iris_engine::renderer::app::run::<Example>()
}
