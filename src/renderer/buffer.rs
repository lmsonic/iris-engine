use std::{fmt::Debug, mem};

use wgpu::util::DeviceExt;

pub struct VertexBuffer<A> {
    pub vertices: Vec<A>,
    // indices: Vec<u32>,
    pub buffer: wgpu::Buffer,
}

impl<A> VertexBuffer<A>
where
    A: Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable,
{
    pub fn new(vertices: Vec<A>, device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });
        // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Index Buffer"),
        //     contents: bytemuck::cast_slice(&indices),
        //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
        // });
        Self {
            vertices,
            // indices,
            buffer,
        }
    }
}
pub struct IndexBuffer {
    pub indices: Vec<u32>,
    // indices: Vec<u32>,
    pub buffer: wgpu::Buffer,
}

impl IndexBuffer {
    pub fn new(indices: Vec<u32>, device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
        });
        // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Index Buffer"),
        //     contents: bytemuck::cast_slice(&indices),
        //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
        // });
        Self {
            indices,
            // indices,
            buffer,
        }
    }
}

pub struct DataBuffer<T> {
    pub data: T,
    pub buffer: wgpu::Buffer,
}

impl<T> DataBuffer<T> {
    pub fn new(data: T, device: &wgpu::Device, usage: wgpu::BufferUsages) -> Self
    where
        T: Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable,
    {
        assert!(
            mem::align_of::<T>() % 4 == 0,
            "Data alignment needs to be multiple of 4"
        );
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[data]),
            usage,
        });
        Self { data, buffer }
    }
    pub fn from_slice<U>(data: T, device: &wgpu::Device, usage: wgpu::BufferUsages) -> Self
    where
        U: Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable,
        T: AsRef<[U]>,
    {
        assert!(
            mem::align_of::<T>() % 4 == 0,
            "Data alignment needs to be multiple of 4"
        );
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(data.as_ref()),
            usage,
        });
        Self { data, buffer }
    }

    pub fn uniform(data: T, device: &wgpu::Device) -> Self
    where
        T: Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable,
    {
        Self::new(
            data,
            device,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        )
    }

    pub fn update(&self, queue: &wgpu::Queue)
    where
        T: Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable,
    {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
    }
}

pub struct Buffer {
    pub buffer: wgpu::Buffer,
}

impl Buffer {
    pub fn new(device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages) -> Self
where {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            usage,
            size,
            mapped_at_creation: false,
        });
        Self { buffer }
    }

    pub fn initialize<T>(self, data: T, queue: &wgpu::Queue) -> DataBuffer<T>
    where
        T: Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable,
    {
        assert!(
            mem::align_of::<T>() % 4 == 0,
            "Data alignment needs to be multiple of 4"
        );

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
        DataBuffer {
            data,
            buffer: self.buffer,
        }
    }
}
