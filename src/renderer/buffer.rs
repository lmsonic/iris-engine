use std::{fmt::Debug, mem};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::GpuSendable;

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
    pub fn new<U>(data: T, device: &wgpu::Device) -> Self
    where
        U: Clone + Copy + Pod + Zeroable,
        T: GpuSendable<U>,
    {
        assert!(
            mem::align_of::<U>() % 4 == 0,
            "Data alignment needs to be multiple of 4"
        );
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[data.to_gpu()]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        Self { data, buffer }
    }

    pub fn update<U>(&self, queue: &wgpu::Queue)
    where
        U: Clone + Copy + Pod + Zeroable,
        T: GpuSendable<U>,
    {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data.to_gpu()]));
    }
}
#[derive(Debug)]
pub struct UniformBufferVec<T> {
    pub data: Vec<T>,
    pub buffer: wgpu::Buffer,
}

impl<T> UniformBufferVec<T> {
    pub fn new<U>(data: &[T], device: &wgpu::Device) -> Self
    where
        U: Clone + Copy + Pod + Zeroable,
        T: GpuSendable<U> + Clone + Copy,
    {
        assert!(
            mem::align_of::<U>() % 4 == 0,
            "Data alignment needs to be multiple of 4"
        );
        let gpu_data: Vec<U> = data.iter().map(|d| d.to_gpu()).collect();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&gpu_data),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        Self {
            data: data.to_vec(),
            buffer,
        }
    }
    pub fn update<U>(&self, queue: &wgpu::Queue)
    where
        U: Clone + Copy + Pod + Zeroable,
        T: GpuSendable<U>,
    {
        let gpu_data: Vec<U> = self.data.iter().map(|d| d.to_gpu()).collect();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&gpu_data));
    }
}

#[derive(Debug)]
pub struct StorageBuffer<T> {
    pub data: T,
    pub buffer: wgpu::Buffer,
}

impl<T> StorageBuffer<T> {
    pub fn new<U>(data: T, device: &wgpu::Device) -> Self
    where
        U: Clone + Copy + Pod + Zeroable,
        T: GpuSendable<U>,
    {
        assert!(
            mem::align_of::<U>() % 4 == 0,
            "Data alignment needs to be multiple of 4"
        );
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[data.to_gpu()]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        Self { data, buffer }
    }

    pub fn update<U>(&self, queue: &wgpu::Queue)
    where
        U: Clone + Copy + Pod + Zeroable,
        T: GpuSendable<U>,
    {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data.to_gpu()]));
    }
}

#[derive(Debug)]
pub struct StorageBufferVec<T> {
    pub data: Vec<T>,
    pub buffer: wgpu::Buffer,
}

impl<T> StorageBufferVec<T> {
    pub fn new<U>(data: &[T], device: &wgpu::Device, queue: &wgpu::Queue, size: u64) -> Self
    where
        U: Clone + Copy + Pod + Zeroable + Debug,
        T: GpuSendable<U> + Clone + Copy,
    {
        assert!(
            mem::align_of::<U>() % 4 == 0,
            "Data alignment needs to be multiple of 4"
        );
        let gpu_data: Vec<U> = data.iter().map(|d| d.to_gpu()).collect();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: mem::size_of::<T>() as u64 * size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&gpu_data));
        Self {
            data: data.to_vec(),
            buffer,
        }
    }
    pub fn update<U>(&self, queue: &wgpu::Queue)
    where
        U: Clone + Copy + Pod + Zeroable,
        T: GpuSendable<U>,
    {
        if self.data.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[U::zeroed()]))
        } else {
            let gpu_data: Vec<U> = self.data.iter().map(|d| d.to_gpu()).collect();
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&gpu_data));
        }
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
