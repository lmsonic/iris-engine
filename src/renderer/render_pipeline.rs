use wgpu::include_wgsl;

use super::resources::VertexAttributeLayout;

#[derive(Debug, Clone, Copy)]
pub struct RenderPipelineWire;

impl<'a> RenderPipelineWire {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> RenderPipelineBuilder<'a> {
        let shader = include_wgsl!("shaders/wire.wgsl");
        RenderPipelineBuilder::new(shader).polygon_mode(wgpu::PolygonMode::Line)
    }
}

#[derive(Debug)]
pub struct RenderPipelineBuilder<'a> {
    shader: wgpu::ShaderModuleDescriptor<'a>,
    depth_texture_format: Option<wgpu::TextureFormat>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    fragment_entry: Option<&'a str>,
    polygon_mode: Option<wgpu::PolygonMode>,
    cull_mode: Option<wgpu::Face>,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(shader: wgpu::ShaderModuleDescriptor<'a>) -> Self {
        Self {
            shader,
            depth_texture_format: Option::default(),
            bind_group_layouts: Vec::default(),
            fragment_entry: Option::default(),
            polygon_mode: Option::default(),
            cull_mode: Option::default(),
        }
    }
    pub fn add_bind_group(mut self, bind_group_layout: &'a wgpu::BindGroupLayout) -> Self {
        self.bind_group_layouts.push(bind_group_layout);
        self
    }
    pub fn depth(self, depth_format: wgpu::TextureFormat) -> Self {
        Self {
            depth_texture_format: Some(depth_format),
            ..self
        }
    }
    pub fn fragment_entry(self, fragment_entry: &'a str) -> Self {
        Self {
            fragment_entry: Some(fragment_entry),
            ..self
        }
    }
    pub fn polygon_mode(self, polygon_mode: wgpu::PolygonMode) -> Self {
        Self {
            polygon_mode: Some(polygon_mode),
            ..self
        }
    }
    pub fn cull_mode(self, cull_mode: Option<wgpu::Face>) -> Self {
        Self { cull_mode, ..self }
    }

    pub fn build<T>(
        self,
        device: &'a wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline
    where
        T: Clone + Copy + bytemuck::Pod + bytemuck::Zeroable + VertexAttributeLayout,
    {
        let module = device.create_shader_module(self.shader);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &self.bind_group_layouts,
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[T::layout()],
                // compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: self.cull_mode,
                polygon_mode: self.polygon_mode.map_or_else(Default::default, |m| m),
                ..Default::default()
            },
            depth_stencil: self.depth_texture_format.map(|depth_texture_format| {
                wgpu::DepthStencilState {
                    format: depth_texture_format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: self.fragment_entry.map_or("fs_main", |f| f),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::all(),
                })],
                // compilation_options: Default::default(),
            }),
            multiview: None,
        })
    }
}

#[derive(Default, Debug)]
pub struct RenderPassBuilder<'a> {
    clear_color: Option<wgpu::Color>,
    depth: Option<&'a wgpu::TextureView>,
}

impl<'a> RenderPassBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub const fn depth(self, depth: &'a wgpu::TextureView) -> Self {
        Self {
            depth: Some(depth),
            ..self
        }
    }
    pub const fn clear_color(self, clear_color: wgpu::Color) -> Self {
        Self {
            clear_color: Some(clear_color),
            ..self
        }
    }
    pub fn build(
        self,
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color.unwrap_or(wgpu::Color::BLACK)),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: self.depth.map(|view| {
                wgpu::RenderPassDepthStencilAttachment {
                    view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }
}
