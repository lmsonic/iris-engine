use bytemuck::{Pod, Zeroable};

pub mod collision;
pub mod renderer;
pub(crate) mod tests;
pub mod visibility;

pub trait GpuSendable<T>
where
    T: Clone + Copy + Pod + Zeroable,
{
    fn to_gpu(&self) -> T;
}

impl<T> GpuSendable<T> for T
where
    T: Clone + Copy + Pod + Zeroable,
{
    fn to_gpu(&self) -> T {
        *self
    }
}
