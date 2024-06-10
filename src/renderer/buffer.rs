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

        Self { indices, buffer }
    }
}

#[derive(Debug)]
pub struct UniformBuffer<T> {
    pub data: T,
    pub buffer: wgpu::Buffer,
}

impl<T> UniformBuffer<T> {
    pub fn new(data: T, device: &wgpu::Device) -> Self
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
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        Self { data, buffer }
    }
    pub fn from_slice<U>(data: T, device: &wgpu::Device) -> Self
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
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        Self { data, buffer }
    }

    pub fn update(&self, queue: &wgpu::Queue)
    where
        T: Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable,
    {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
    }
}

#[derive(Debug)]
pub struct StorageBuffer<T> {
    pub data: T,
    pub buffer: wgpu::Buffer,
}

impl<T> StorageBuffer<T> {
    pub fn new(data: T, device: &wgpu::Device) -> Self
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
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        Self { data, buffer }
    }
    pub fn from_slice<U>(data: T, device: &wgpu::Device) -> Self
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
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        Self { data, buffer }
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
    pub fn new(device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            usage,
            size,
            mapped_at_creation: false,
        });
        Self { buffer }
    }
}
