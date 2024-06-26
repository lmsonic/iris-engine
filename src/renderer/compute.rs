pub fn generate_mipmaps(
    texture: &wgpu::Texture,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layer: u32,
) {
    // Create mip views and sizes
    let mut mip_sizes = vec![texture.size()];
    let mut mip_views = vec![];
    let mip_level_count = texture.mip_level_count();
    for level in 0..mip_level_count {
        mip_views.push(texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("mip view: {level}")),
            format: Some(texture.format()),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: level,
            mip_level_count: Some(1),
            base_array_layer: layer,
            array_layer_count: Some(texture.depth_or_array_layers()),
        }));
        if level > 0 {
            let previous_size = mip_sizes[level as usize - 1];
            mip_sizes.push(wgpu::Extent3d {
                width: previous_size.width / 2,
                height: previous_size.height / 2,
                depth_or_array_layers: previous_size.depth_or_array_layers / 2,
            });
        }
    }

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });

    // Create bind groups in advance because of rust borrow rules
    let mut bind_groups = vec![];
    for level in 1..mip_level_count {
        bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&mip_views[level as usize - 1]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&mip_views[level as usize]),
                },
            ],
        }));
    }

    let shader = wgpu::include_wgsl!("shaders/mipmap_generation.wgsl");

    let compute_pipeline = ComputePipelineBuilder::new(shader)
        .add_bind_group(&bind_group_layout)
        .build(device, "compute_mip_map");

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Compute Pass"),
        timestamp_writes: None,
    });
    compute_pass.set_pipeline(&compute_pipeline);

    for level in 1..mip_level_count {
        // We write to each mip level using the previous level
        let level = level as usize;
        compute_pass.set_bind_group(0, &bind_groups[level - 1], &[]);
        let invocation_count_x = mip_sizes[level - 1].width;
        let invocation_count_y = mip_sizes[level - 1].height;
        let workgroup_size_per_dim = 8;
        // This ceils invocation_count / workgroup_size
        let workgroup_count_x =
            (invocation_count_x + workgroup_size_per_dim - 1) / workgroup_size_per_dim;
        let workgroup_count_y =
            (invocation_count_y + workgroup_size_per_dim - 1) / workgroup_size_per_dim;
        compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
    }

    drop(compute_pass);

    let command = encoder.finish();

    queue.submit([command]);

    // for level in 1..mip_level_count {
    //     save_texture(
    //         format!(
    //             "{}_mip{level}.png",
    //             path.as_ref().with_extension("").display()
    //         ),
    //         &texture,
    //         device,
    //         queue,
    //         level,
    //     );
    // }
}

#[derive(Debug)]
pub struct ComputePipelineBuilder<'a> {
    shader: wgpu::ShaderModuleDescriptor<'a>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
}

impl<'a> ComputePipelineBuilder<'a> {
    pub fn new(shader: wgpu::ShaderModuleDescriptor<'a>) -> Self {
        Self {
            shader,
            bind_group_layouts: vec![],
        }
    }

    pub fn add_bind_group(mut self, bind_group_layout: &'a wgpu::BindGroupLayout) -> Self {
        self.bind_group_layouts.push(bind_group_layout);
        self
    }

    pub fn build(self, device: &'a wgpu::Device, entry_point: &str) -> wgpu::ComputePipeline {
        let module = device.create_shader_module(self.shader);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &self.bind_group_layouts,
            push_constant_ranges: &[],
        });

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point,
        })
    }
}
