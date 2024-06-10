use wgpu::include_wgsl;

use super::resources::VertexAttributeLayout;

pub struct RenderPipelineWire;

impl RenderPipelineWire {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> RenderPipelineBuilder {
        let shader = include_wgsl!("wire.wgsl");
        RenderPipelineBuilder::new(device, shader, surface_format)
            .polygon_mode(wgpu::PolygonMode::Line)
    }
}

pub struct RenderPipelineBuilder<'a> {
    device: &'a wgpu::Device,
    shader_module: wgpu::ShaderModule,
    surface_format: wgpu::TextureFormat,
    depth_texture_format: Option<wgpu::TextureFormat>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    fragment_entry: Option<&'a str>,
    polygon_mode: Option<wgpu::PolygonMode>,
    cull_mode: Option<wgpu::Face>,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(
        device: &'a wgpu::Device,
        shader: wgpu::ShaderModuleDescriptor,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader_module = device.create_shader_module(shader);

        Self {
            device,
            shader_module,
            surface_format,
            bind_group_layouts: vec![],
            depth_texture_format: None,
            fragment_entry: None,
            polygon_mode: None,
            cull_mode: Some(wgpu::Face::Back),
        }
    }
    pub fn add_bind_group(mut self, bind_group_layout: &'a wgpu::BindGroupLayout) -> Self {
        self.bind_group_layouts.push(bind_group_layout);
        Self {
            bind_group_layouts: self.bind_group_layouts,
            depth_texture_format: self.depth_texture_format,
            device: self.device,
            shader_module: self.shader_module,
            surface_format: self.surface_format,
            fragment_entry: self.fragment_entry,
            polygon_mode: self.polygon_mode,
            cull_mode: self.cull_mode,
        }
    }
    pub fn depth(self, depth_format: wgpu::TextureFormat) -> Self {
        Self {
            depth_texture_format: Some(depth_format),
            bind_group_layouts: self.bind_group_layouts,
            device: self.device,
            shader_module: self.shader_module,
            surface_format: self.surface_format,
            fragment_entry: self.fragment_entry,
            polygon_mode: self.polygon_mode,
            cull_mode: self.cull_mode,
        }
    }
    pub fn fragment_entry(self, fragment_entry: &'a str) -> RenderPipelineBuilder<'a> {
        Self {
            fragment_entry: Some(fragment_entry),
            depth_texture_format: self.depth_texture_format,
            bind_group_layouts: self.bind_group_layouts,
            device: self.device,
            shader_module: self.shader_module,
            surface_format: self.surface_format,
            polygon_mode: self.polygon_mode,
            cull_mode: self.cull_mode,
        }
    }
    pub fn polygon_mode(self, polygon_mode: wgpu::PolygonMode) -> Self {
        Self {
            fragment_entry: self.fragment_entry,
            depth_texture_format: self.depth_texture_format,
            bind_group_layouts: self.bind_group_layouts,
            device: self.device,
            shader_module: self.shader_module,
            surface_format: self.surface_format,
            polygon_mode: Some(polygon_mode),
            cull_mode: self.cull_mode,
        }
    }
    pub fn cull_mode(self, cull_mode: Option<wgpu::Face>) -> Self {
        Self {
            fragment_entry: self.fragment_entry,
            depth_texture_format: self.depth_texture_format,
            bind_group_layouts: self.bind_group_layouts,
            device: self.device,
            shader_module: self.shader_module,
            surface_format: self.surface_format,
            polygon_mode: self.polygon_mode,
            cull_mode,
        }
    }

    pub fn build<T>(self) -> wgpu::RenderPipeline
    where
        T: Clone + Copy + bytemuck::Pod + bytemuck::Zeroable + VertexAttributeLayout,
    {
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &self.bind_group_layouts,
                push_constant_ranges: &[],
            });

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader_module,
                    entry_point: "vs_main",
                    buffers: &[T::layout()],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: self.cull_mode,
                    polygon_mode: self.polygon_mode.map_or(Default::default(), |m| m),
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
                    module: &self.shader_module,
                    entry_point: self.fragment_entry.map_or("fs_main", |f| f),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.surface_format,
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
                    compilation_options: Default::default(),
                }),
                multiview: None,
            })
    }
}

pub struct RenderPassBuilder<'a> {
    clear_color: wgpu::Color,
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
    depth: Option<&'a wgpu::TextureView>,
}

impl<'a> RenderPassBuilder<'a> {
    pub fn new(encoder: &'a mut wgpu::CommandEncoder, view: &'a wgpu::TextureView) -> Self {
        Self {
            clear_color: wgpu::Color::default(),
            encoder,
            view,
            depth: None,
        }
    }
    pub fn depth(self, depth: &'a wgpu::TextureView) -> Self {
        Self {
            clear_color: self.clear_color,
            encoder: self.encoder,
            view: self.view,
            depth: Some(depth),
        }
    }
    pub fn clear_color(self, clear_color: wgpu::Color) -> Self {
        Self {
            clear_color,
            encoder: self.encoder,
            view: self.view,
            depth: self.depth,
        }
    }
    pub fn build(self) -> wgpu::RenderPass<'a> {
        self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
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
