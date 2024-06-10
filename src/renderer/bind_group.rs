use super::texture::Texture;

pub struct BindGroup {
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

pub struct BindGroupBuilder<'a> {
    counter: u32,
    layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    bind_group_entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            layout_entries: vec![],
            bind_group_entries: vec![],
        }
    }
    pub fn uniform(mut self, uniform_buffer: &'a wgpu::Buffer) -> Self {
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.counter,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        self.bind_group_entries.push(wgpu::BindGroupEntry {
            binding: self.counter,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: uniform_buffer,
                offset: 0,
                size: None,
            }),
        });
        self.counter += 1;

        self
    }

    pub fn storage_buffer(mut self, storage_buffer: &'a wgpu::Buffer) -> Self {
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.counter,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        self.bind_group_entries.push(wgpu::BindGroupEntry {
            binding: self.counter,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: storage_buffer,
                offset: 0,
                size: None,
            }),
        });
        self.counter += 1;

        self
    }

    pub fn texture(mut self, texture: &'a Texture) -> BindGroupBuilder<'a> {
        self.layout_entries.extend([
            wgpu::BindGroupLayoutEntry {
                binding: self.counter,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: self.counter + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]);

        self.bind_group_entries.extend([
            wgpu::BindGroupEntry {
                binding: self.counter,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: self.counter + 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            },
        ]);
        self.counter += 2;

        self
    }

    pub fn build(self, device: &wgpu::Device) -> BindGroup {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &self.layout_entries,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group Layout"),

            layout: &bind_group_layout,
            entries: &self.bind_group_entries,
        });
        BindGroup {
            layout: bind_group_layout,
            bind_group,
        }
    }
}

impl<'a> Default for BindGroupBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}
