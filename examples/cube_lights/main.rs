use bytemuck::{Pod, Zeroable};

use glam::{Mat4, Vec3};
use iris_engine::{
    geometry::shapes::Cuboid,
    renderer::{
        color::Color,
        light::{DirectionalLight, PointLight},
        mesh::{Meshable, Vertex},
    },
};
use std::{f32::consts, mem};
use wgpu::{include_wgsl, util::DeviceExt, vertex_attr_array};
use winit::{event::KeyEvent, keyboard::PhysicalKey};

struct Example {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
    camera: Camera,
}

#[derive(Debug, Clone, Copy)]
struct Camera {
    aspect_ratio: f32,
    position: Vec3,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct CameraGPU {
    view_proj: Mat4,
    position: Vec3,
    _pad: f32,
}

impl Camera {
    fn new(aspect_ratio: f32, position: Vec3) -> Self {
        Self {
            aspect_ratio,
            position,
        }
    }
    fn matrix(&self) -> CameraGPU {
        let projection =
            glam::Mat4::perspective_rh(consts::FRAC_PI_4, self.aspect_ratio, 1.0, 10.0);
        let center = Vec3::ZERO;
        let up = Vec3::Y;
        let view = glam::Mat4::look_at_rh(self.position, center, up);
        CameraGPU {
            view_proj: projection * view,
            position: self.position,
            _pad: Default::default(),
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
        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<Vertex>();
        let cube = Cuboid::new(Vec3::splat(1.0)).mesh();
        let vertices = cube.vertices();
        let indices = cube.indices();
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        // Create other resources
        let camera = Camera::new(config.width as f32 / config.height as f32, Vec3::splat(2.0));

        let camera_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera.matrix()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let directional_light = DirectionalLight::new(Color::WHITE, Vec3::new(0.0, -1.0, 0.0));

        let directional_lights_uniform =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light Uniform Buffer"),
                contents: bytemuck::cast_slice(&[directional_light]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let point_light = PointLight::new(
            Color::WHITE,
            Vec3::new(1.0, 1.0, 0.0),
            10.0,
            [0.0, 2.0, 0.0],
        );

        let point_lights_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Uniform Buffer"),
            contents: bytemuck::cast_slice(&[point_light]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        // Create bind group
        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: directional_lights_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: point_lights_uniform.as_entire_binding(),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float32x4,1 => Float32x3],
        }];
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(config.view_formats[0].into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let pipeline_wire = if device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            let pipeline_wire = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    compilation_options: Default::default(),
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_wire",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.view_formats[0],
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                operation: wgpu::BlendOperation::Add,
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            },
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Line,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
            Some(pipeline_wire)
        } else {
            None
        };

        // Done
        Example {
            vertex_buf,
            index_buf,
            index_count: indices.len(),
            bind_group,
            uniform_buf: camera_uniform,
            pipeline,
            pipeline_wire,
            camera,
        }
    }

    fn update(&mut self, event: winit::event::WindowEvent) {
        //empty
        if let winit::event::WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    ..
                },
            ..
        } = event
        {
            match code {
                winit::keyboard::KeyCode::KeyA => self.camera.position.x -= 0.1,
                winit::keyboard::KeyCode::KeyD => self.camera.position.x += 0.1,
                winit::keyboard::KeyCode::ShiftLeft => self.camera.position.y -= 0.1,
                winit::keyboard::KeyCode::Space => self.camera.position.y += 0.1,
                winit::keyboard::KeyCode::KeyW => self.camera.position.z -= 0.1,
                winit::keyboard::KeyCode::KeyS => self.camera.position.z += 0.1,
                _ => {}
            }
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.camera.aspect_ratio = config.width as f32 / config.height as f32;

        queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::cast_slice(&[self.camera.matrix()]),
        );
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::cast_slice(&[self.camera.matrix()]),
        );

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
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
            if let Some(ref pipe) = self.pipeline_wire {
                rpass.set_pipeline(pipe);
                rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}

pub fn main() -> Result<(), winit::error::EventLoopError> {
    iris_engine::renderer::app::run::<Example>()
}
