use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use glam::{Affine3A, Quat, Vec3};

use super::aabb::Aabb;

#[derive(Debug, Clone, Copy)]
pub struct Obb {
    // Rotation gets applied in the local space of Aabb
    pub rotation: Quat,
    pub aabb: Aabb,
}

impl Obb {
    pub const fn new(rotation: Quat, center: Vec3, size: Vec3) -> Self {
        Self {
            rotation,
            aabb: Aabb { center, size },
        }
    }
    pub const fn center(&self) -> Vec3 {
        self.aabb.center
    }

    pub const fn size(&self) -> Vec3 {
        self.aabb.size
    }

    pub fn min(&self) -> Vec3 {
        self.aabb.min()
    }
    pub fn max(&self) -> Vec3 {
        self.aabb.max()
    }

    pub fn contains(&self, point: Vec3) -> bool {
        let point = self.transform_point(point);
        self.aabb.contains(point)
    }

    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        let center = self.center();
        self.rotation.inverse() * (point - center)
    }

    pub fn intersect_aabb(&self, other: Aabb) -> bool {
        let min = self.transform_point(other.min());
        let max = self.transform_point(other.max());
        let other = Aabb::from_min_max(min, max);
        self.aabb.intersect_aabb(other)
    }
    pub fn contains_aabb(&self, other: Aabb) -> bool {
        let min = self.transform_point(other.min());
        let max = self.transform_point(other.max());
        let other = Aabb::from_min_max(min, max);
        self.aabb.contains_aabb(other)
    }
}

// Translation operations
impl AddAssign<Vec3> for Obb {
    fn add_assign(&mut self, rhs: Vec3) {
        *self = *self + rhs;
    }
}
impl SubAssign<Vec3> for Obb {
    fn sub_assign(&mut self, rhs: Vec3) {
        *self = *self - rhs;
    }
}

impl Add<Vec3> for Obb {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            aabb: self.aabb + rhs,
            rotation: self.rotation,
        }
    }
}
impl Add<Obb> for Vec3 {
    type Output = Obb;

    fn add(self, rhs: Obb) -> Self::Output {
        Self::Output {
            aabb: rhs.aabb + self,
            rotation: rhs.rotation,
        }
    }
}

impl Sub<Vec3> for Obb {
    type Output = Self;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            aabb: self.aabb - rhs,
            rotation: self.rotation,
        }
    }
}

// Scale operations
impl MulAssign<Vec3> for Obb {
    fn mul_assign(&mut self, rhs: Vec3) {
        *self = *self * rhs;
    }
}

impl Mul<Vec3> for Obb {
    type Output = Self;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            aabb: self.aabb * rhs,

            rotation: self.rotation,
        }
    }
}
impl Mul<Obb> for Vec3 {
    type Output = Obb;

    fn mul(self, rhs: Obb) -> Self::Output {
        Self::Output {
            aabb: rhs.aabb * self,
            rotation: rhs.rotation,
        }
    }
}

impl MulAssign<f32> for Obb {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Mul<f32> for Obb {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            aabb: self.aabb * rhs,
            rotation: self.rotation,
        }
    }
}
impl Mul<Obb> for f32 {
    type Output = Obb;

    fn mul(self, rhs: Obb) -> Self::Output {
        Self::Output {
            aabb: rhs.aabb * self,
            rotation: rhs.rotation,
        }
    }
}
// Rotation operations
impl Mul<Obb> for Quat {
    type Output = Obb;

    fn mul(self, rhs: Obb) -> Self::Output {
        Self::Output {
            rotation: self * rhs.rotation,
            aabb: rhs.aabb,
        }
    }
}
// Multiply with transform
impl Mul<Obb> for Affine3A {
    type Output = Obb;

    fn mul(self, rhs: Obb) -> Self::Output {
        let (scale, rotation, translation) = self.to_scale_rotation_translation();
        Self::Output {
            rotation: rotation * rhs.rotation,
            aabb: scale * rhs.aabb + translation,
        }
    }
}

impl From<Aabb> for Obb {
    fn from(value: Aabb) -> Self {
        Self {
            rotation: Quat::IDENTITY,
            aabb: value,
        }
    }
}
