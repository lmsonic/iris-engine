use wgpu::BindGroupEntry;

pub trait AsBindGroup {
    type Data;
    fn label() -> Option<&'static str> {
        None
    }
    fn as_bind_group(&self, device: &wgpu::Device) -> BindGroup<Self::Data> {
        let layout = Self::bind_group_layout(device);
        let bindings = self.bindings(device, &layout);
        let data = self.data();
        let entries = bindings
            .iter()
            .map(|(index, binding)| BindGroupEntry {
                binding: *index,
                resource: binding.get_binding(),
            })
            .collect::<Vec<_>>();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Self::label(),
            layout: &layout,
            entries: &entries,
        });

        BindGroup::new(layout, bind_group, data)
    }

    fn data(&self) -> Self::Data;

    fn bindings(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> Vec<(u32, OwnedBindingResource)>;

    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;
}

#[derive(Debug)]
pub enum OwnedBindingResource {
    Buffer(wgpu::Buffer),
    TextureView(wgpu::TextureView),
    Sampler(wgpu::Sampler),
}

impl OwnedBindingResource {
    pub fn get_binding(&self) -> wgpu::BindingResource {
        match self {
            Self::Buffer(buffer) => buffer.as_entire_binding(),
            Self::TextureView(view) => wgpu::BindingResource::TextureView(view),
            Self::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
        }
    }
}

#[derive(Debug)]
pub struct BindGroup<T> {
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    data: T,
}

impl<T> BindGroup<T> {
    fn new(layout: wgpu::BindGroupLayout, bind_group: wgpu::BindGroup, data: T) -> Self {
        Self {
            layout,
            bind_group,
            data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindGroupLayoutBuilder {
    counter: u32,
    layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    label: Option<String>,
    visibility: wgpu::ShaderStages,
}

impl BindGroupLayoutBuilder {
    pub const fn new(visibility: wgpu::ShaderStages) -> Self {
        Self {
            counter: 0,
            layout_entries: vec![],
            label: None,
            visibility,
        }
    }

    pub fn label(&mut self, label: &str) {
        self.label = Some(label.to_owned());
    }

    pub fn buffer(&mut self, ty: wgpu::BufferBindingType) -> &mut Self {
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.counter,
            visibility: self.visibility,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        self.counter += 1;
        self
    }
    pub fn uniform(&mut self) -> &mut Self {
        self.buffer(wgpu::BufferBindingType::Uniform)
    }
    pub fn storage(&mut self, read_only: bool) -> &mut Self {
        self.buffer(wgpu::BufferBindingType::Storage { read_only })
    }
    pub fn storage_texture(
        &mut self,
        access: wgpu::StorageTextureAccess,
        format: wgpu::TextureFormat,
        view_dimension: wgpu::TextureViewDimension,
    ) -> &mut Self {
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: self.visibility,
            ty: wgpu::BindingType::StorageTexture {
                access,
                format,
                view_dimension,
            },
            count: None,
        });
        self.counter += 1;
        self
    }

    pub fn texture(
        &mut self,
        view_dimension: wgpu::TextureViewDimension,
        sample_type: wgpu::TextureSampleType,
    ) -> &mut Self {
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.counter,
            visibility: self.visibility,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension,
                multisampled: false,
            },
            count: None,
        });
        self.counter += 1;
        self
    }
    pub fn texture_2d(&mut self) -> &mut Self {
        self.texture(
            wgpu::TextureViewDimension::D2,
            wgpu::TextureSampleType::Float { filterable: true },
        )
    }
    pub fn sampler(&mut self, sampler_type: wgpu::SamplerBindingType) -> &mut Self {
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.counter,
            visibility: self.visibility,
            ty: wgpu::BindingType::Sampler(sampler_type),
            count: None,
        });
        self.counter += 1;
        self
    }

    pub fn build(&mut self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: self.label.as_deref(),
            entries: &self.layout_entries,
        })
    }

    pub fn visibility(&mut self, visibility: wgpu::ShaderStages) {
        self.visibility = visibility;
    }
}
