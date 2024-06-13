use std::ops::{Add, AddAssign, Mul, MulAssign};

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

#[allow(clippy::module_name_repetitions)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
impl From<wgpu::Color> for Color {
    fn from(value: wgpu::Color) -> Self {
        Color::new(value.r as f32, value.g as f32, value.b as f32)
    }
}
impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        wgpu::Color {
            r: value.r.into(),
            g: value.g.into(),
            b: value.b.into(),
            a: 1.0,
        }
    }
}
impl From<[f32; 3]> for Color {
    fn from(value: [f32; 3]) -> Self {
        Color::new(value[0], value[1], value[2])
    }
}
impl From<Color> for [f32; 3] {
    fn from(value: Color) -> Self {
        [value.r, value.g, value.b]
    }
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Color::new(value.x, value.y, value.z)
    }
}
impl From<Color> for Vec3 {
    fn from(value: Color) -> Self {
        Vec3::new(value.r, value.g, value.b)
    }
}

impl Color {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0);
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}
impl MulAssign for Color {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: rhs * self.r,
            g: rhs * self.g,
            b: rhs * self.b,
        }
    }
}
impl Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}
