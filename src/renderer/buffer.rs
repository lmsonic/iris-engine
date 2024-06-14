use std::{fmt::Debug, mem};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::GpuSendable;

#[derive(Debug)]
pub struct VertexBuffer<A> {
    pub vertices: Vec<A>,
    pub buffer: wgpu::Buffer,
}

impl<A> VertexBuffer<A>
where
    A: Debug + Clone + Copy + Pod + Zeroable,
{
    pub fn new(vertices: Vec<A>, device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });
        Self { vertices, buffer }
    }
}
#[derive(Debug)]
pub struct IndexBuffer {
    pub indices: Vec<u32>,
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
pub struct UniformBufferArray<T> {
    pub data: Vec<T>,
    pub buffer: wgpu::Buffer,
}

impl<T> UniformBufferArray<T> {
    pub fn new<U>(data: &[T], device: &wgpu::Device) -> Self
    where
        U: Clone + Copy + Pod + Zeroable,
        T: GpuSendable<U> + Clone + Copy,
    {
        let gpu_data: Vec<U> = data.iter().map(GpuSendable::to_gpu).collect();
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
        let gpu_data: Vec<U> = self.data.iter().map(GpuSendable::to_gpu).collect();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&gpu_data));
    }
    pub fn update_at<U>(&self, index: usize, queue: &wgpu::Queue)
    where
        U: Clone + Copy + Pod + Zeroable,
        T: GpuSendable<U>,
    {
        assert!(index < self.data.len(), "Updated buffer at index > length");

        let gpu_data = self.data[index].to_gpu();
        queue.write_buffer(
            &self.buffer,
            index as u64,
            bytemuck::cast_slice(&[gpu_data]),
        );
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
pub struct StorageBufferArray<T> {
    pub data: Vec<T>,
    pub buffer: wgpu::Buffer,
}

impl<T> StorageBufferArray<T> {
    pub fn new<U>(data: &[T], device: &wgpu::Device, queue: &wgpu::Queue, size: u64) -> Self
    where
        U: Clone + Copy + Pod + Zeroable + Debug,
        T: GpuSendable<U> + Clone + Copy,
    {
        let gpu_data: Vec<U> = data.iter().map(GpuSendable::to_gpu).collect();
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
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[U::zeroed()]));
        } else {
            let gpu_data: Vec<U> = self.data.iter().map(GpuSendable::to_gpu).collect();
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&gpu_data));
        }
    }
    pub fn update_at<U>(&self, index: usize, queue: &wgpu::Queue)
    where
        U: Clone + Copy + Pod + Zeroable,
        T: GpuSendable<U>,
    {
        assert!(index < self.data.len(), "Updated buffer at index > length");

        let gpu_data = self.data[index].to_gpu();
        queue.write_buffer(
            &self.buffer,
            index as u64,
            bytemuck::cast_slice(&[gpu_data]),
        );
    }
}
#[derive(Debug)]
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
