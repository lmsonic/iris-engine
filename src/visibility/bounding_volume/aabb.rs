use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use glam::{Affine3A, Quat, Vec2, Vec3};

use crate::renderer::mesh::{Mesh, Meshable, Vertex};

use super::obb::Obb;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub center: Vec3,
    pub size: Vec3,
}
impl Meshable for Aabb {
    fn mesh(&self) -> Mesh {
        let min = self.min();
        let max = self.max();

        // Suppose Y-up right hand, and camera look from +Z to -Z
        let vertices = &[
            // Front
            ([min.x, min.y, max.z], [0.0, 0.0, 1.0], [0.0, 0.0]),
            ([max.x, min.y, max.z], [0.0, 0.0, 1.0], [1.0, 0.0]),
            ([max.x, max.y, max.z], [0.0, 0.0, 1.0], [1.0, 1.0]),
            ([min.x, max.y, max.z], [0.0, 0.0, 1.0], [0.0, 1.0]),
            // Back
            ([min.x, max.y, min.z], [0.0, 0.0, -1.0], [1.0, 0.0]),
            ([max.x, max.y, min.z], [0.0, 0.0, -1.0], [0.0, 0.0]),
            ([max.x, min.y, min.z], [0.0, 0.0, -1.0], [0.0, 1.0]),
            ([min.x, min.y, min.z], [0.0, 0.0, -1.0], [1.0, 1.0]),
            // Right
            ([max.x, min.y, min.z], [1.0, 0.0, 0.0], [0.0, 0.0]),
            ([max.x, max.y, min.z], [1.0, 0.0, 0.0], [1.0, 0.0]),
            ([max.x, max.y, max.z], [1.0, 0.0, 0.0], [1.0, 1.0]),
            ([max.x, min.y, max.z], [1.0, 0.0, 0.0], [0.0, 1.0]),
            // Left
            ([min.x, min.y, max.z], [-1.0, 0.0, 0.0], [1.0, 0.0]),
            ([min.x, max.y, max.z], [-1.0, 0.0, 0.0], [0.0, 0.0]),
            ([min.x, max.y, min.z], [-1.0, 0.0, 0.0], [0.0, 1.0]),
            ([min.x, min.y, min.z], [-1.0, 0.0, 0.0], [1.0, 1.0]),
            // Top
            ([max.x, max.y, min.z], [0.0, 1.0, 0.0], [1.0, 0.0]),
            ([min.x, max.y, min.z], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([min.x, max.y, max.z], [0.0, 1.0, 0.0], [0.0, 1.0]),
            ([max.x, max.y, max.z], [0.0, 1.0, 0.0], [1.0, 1.0]),
            // Bottom
            ([max.x, min.y, max.z], [0.0, -1.0, 0.0], [0.0, 0.0]),
            ([min.x, min.y, max.z], [0.0, -1.0, 0.0], [1.0, 0.0]),
            ([min.x, min.y, min.z], [0.0, -1.0, 0.0], [1.0, 1.0]),
            ([max.x, min.y, min.z], [0.0, -1.0, 0.0], [0.0, 1.0]),
        ];

        let vertices: Vec<Vertex> = vertices
            .iter()
            .map(|&(p, n, u)| Vertex {
                position: Vec3::from(p),
                normal: Vec3::from(n),
                uv: Vec2::from(u),
                ..Default::default()
            })
            .collect();

        let indices = vec![
            0, 1, 2, 2, 3, 0, // front
            4, 5, 6, 6, 7, 4, // back
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // top
            20, 21, 22, 22, 23, 20, // bottom
        ];
        Mesh::new(vertices, indices)
    }
}

impl Aabb {
    pub const fn new(center: Vec3, size: Vec3) -> Self {
        Self { center, size }
    }
    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        let center = (max + min) * 0.5;
        let size = (max - min) * 0.5;
        Self { center, size }
    }
    pub fn min(&self) -> Vec3 {
        self.center - self.size * 0.5
    }
    pub fn max(&self) -> Vec3 {
        self.center + self.size * 0.5
    }

    pub fn contains(&self, point: Vec3) -> bool {
        let min = self.min();
        let max = self.max();
        point.cmpge(min).all() && point.cmple(max).all()
    }
    pub fn closest_point_on_aabb(&self, point: Vec3) -> Vec3 {
        let min = self.min();
        let max = self.max();
        point.clamp(min, max)
    }

    pub fn contains_aabb(&self, other: Self) -> bool {
        let other_min = other.min();
        let other_max = other.max();
        let min = self.min();
        let max = self.max();
        other_min.x >= min.x
            && other_max.x <= max.x
            && other_min.y >= min.y
            && other_max.y <= max.y
            && other_min.z >= min.z
            && other_max.z <= max.z
    }
    pub fn intersect_aabb(&self, other: Self) -> bool {
        let other_min = other.min();
        let other_max = other.max();
        let min = self.min();
        let max = self.max();
        other_min.x <= max.x
            && other_max.x >= min.x
            && other_min.y <= max.y
            && other_max.y >= min.y
            && other_min.z <= max.z
            && other_max.z >= min.z
    }
    pub fn contains_obb(&self, other: Obb) -> bool {
        let min = other.transform_point(self.min());
        let max = other.transform_point(self.max());
        let self_transformed = Self::from_min_max(min, max);
        self_transformed.contains_aabb(other.aabb)
    }
    pub fn intersect_obb(&self, other: Obb) -> bool {
        other.intersect_aabb(*self)
    }
    pub fn from_points(points: &[Vec3]) -> Self {
        // Transform all points by rotation
        let first = points
            .iter()
            .next()
            .expect("mesh must contain at least one point for Aabb construction");

        let (min, max) = points
            .iter()
            .fold((*first, *first), |(prev_min, prev_max), point| {
                (point.min(prev_min), point.max(prev_max))
            });

        Self::from_min_max(min, max)
    }
}

// Translation operations
impl AddAssign<Vec3> for Aabb {
    fn add_assign(&mut self, rhs: Vec3) {
        *self = *self + rhs;
    }
}
impl SubAssign<Vec3> for Aabb {
    fn sub_assign(&mut self, rhs: Vec3) {
        *self = *self - rhs;
    }
}

impl Add<Vec3> for Aabb {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            center: self.center + rhs,
            size: self.size,
        }
    }
}
impl Add<Aabb> for Vec3 {
    type Output = Aabb;

    fn add(self, rhs: Aabb) -> Self::Output {
        Self::Output {
            center: rhs.center + self,
            size: rhs.size,
        }
    }
}

impl Sub<Vec3> for Aabb {
    type Output = Self;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            center: self.center - rhs,
            size: self.size,
        }
    }
}

// Scale operations
impl MulAssign<Vec3> for Aabb {
    fn mul_assign(&mut self, rhs: Vec3) {
        *self = *self * rhs;
    }
}

impl Mul<Vec3> for Aabb {
    type Output = Self;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            center: self.center,
            size: self.size * rhs,
        }
    }
}
impl Mul<Aabb> for Vec3 {
    type Output = Aabb;

    fn mul(self, rhs: Aabb) -> Self::Output {
        Self::Output {
            center: rhs.center,
            size: rhs.size * self,
        }
    }
}

impl MulAssign<f32> for Aabb {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Mul<f32> for Aabb {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            center: self.center,
            size: self.size * rhs,
        }
    }
}
impl Mul<Aabb> for f32 {
    type Output = Aabb;

    fn mul(self, rhs: Aabb) -> Self::Output {
        Self::Output {
            center: rhs.center,
            size: rhs.size * self,
        }
    }
}
// Rotation operations
impl Mul<Aabb> for Quat {
    type Output = Obb;

    fn mul(self, rhs: Aabb) -> Self::Output {
        Self::Output {
            rotation: self,
            aabb: rhs,
        }
    }
}
// Multiply with transform
impl Mul<Aabb> for Affine3A {
    type Output = Obb;

    fn mul(self, rhs: Aabb) -> Self::Output {
        let (scale, rotation, translation) = self.to_scale_rotation_translation();
        Self::Output {
            rotation,
            aabb: Aabb {
                center: rhs.center + translation,
                size: rhs.size * scale,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;

    use glam::{Quat, Vec3};

    use crate::{
        tests::{any_normal, any_quat, any_vec3},
        visibility::bounding_volume::obb::Obb,
    };

    use super::Aabb;
    const SIZE_RANGE: RangeInclusive<f32> = 0.1..=100.0;
    const RANGE: RangeInclusive<f32> = -100.0..=100.0;
    use proptest::proptest;
    proptest! {
        #[test]
        fn aabb_contains(center in any_vec3(RANGE),size in any_vec3(SIZE_RANGE),offset in any_vec3(0.01..=0.5_f32)){
            _aabb_contains(center, size,center+ offset*size);
        }
        #[test]
        fn aabb_not_contains(center in any_vec3(RANGE),size in any_vec3(SIZE_RANGE),offset in any_vec3(1.1..=100.0_f32)){
            _aabb_not_contains(center, size,center+ offset*size);
        }
    }
    fn _aabb_contains(center: Vec3, size: Vec3, point: Vec3) {
        let aabb = Aabb::new(center, size);
        assert!(aabb.contains(point));
    }
    fn _aabb_not_contains(center: Vec3, size: Vec3, point: Vec3) {
        let aabb = Aabb::new(center, size);
        assert!(!aabb.contains(point));
    }

    proptest! {
        #[test]
        fn aabb_contains_smaller_with_same_center(size in any_vec3(SIZE_RANGE),factor in 0.01..=0.99_f32,center in any_vec3(RANGE)){
            let smaller_size = factor*size;
            _aabb_contains_smaller_with_same_center(size, smaller_size, center);
        }
        // #[test]
        // fn aabb_intersect_with_same_center(size1 in any_vec3(SIZE_RANGE),size2 in any_vec3(SIZE_RANGE),center in any_vec3(RANGE)){
        //     _aabb_intersect_with_same_center(size1, size2, center);
        // }


        // #[test]
        // fn aabb_intersect_obb_with_same_center(size1 in any_vec3(SIZE_RANGE),size2 in any_vec3(SIZE_RANGE),center in any_vec3(RANGE),rotation in any_quat()){
        //     _aabb_intersect_obb_with_same_center(size1, size2, center,rotation);
        // }

        #[test]
        fn aabb_does_not_intersect_with_distance_larger_than_sum_of_sizes(size1 in any_vec3(SIZE_RANGE),size2 in any_vec3(SIZE_RANGE),direction in any_normal(),center in any_vec3(RANGE)){
            _aabb_does_not_intersect_with_distance_larger_than_sum_of_sizes(size1,size2,direction,center);
        }
    }

    fn _aabb_contains_smaller_with_same_center(
        bigger_size: Vec3,
        smaller_size: Vec3,
        center: Vec3,
    ) {
        let bigger = Aabb::new(center, bigger_size);
        let smaller = Aabb::new(center, smaller_size);
        assert!(bigger.contains_aabb(smaller));
        assert!(bigger.intersect_aabb(smaller));
    }

    fn _aabb_intersect_with_same_center(size1: Vec3, size2: Vec3, center: Vec3) {
        let b1 = Aabb::new(center, size1);
        let b2 = Aabb::new(center, size2);
        assert!(b1.intersect_aabb(b2));
        let b2 = Obb::new(Quat::IDENTITY, center, size2);
        assert!(b1.intersect_obb(b2));
    }
    fn _aabb_intersect_obb_with_same_center(
        size1: Vec3,
        size2: Vec3,
        center: Vec3,
        rotation: Quat,
    ) {
        let b1 = Aabb::new(center, size1);
        let b2 = Obb::new(rotation, center, size2);
        assert!(b1.intersect_obb(b2));
    }

    fn _aabb_does_not_intersect_with_distance_larger_than_sum_of_sizes(
        size1: Vec3,
        size2: Vec3,
        direction: Vec3,
        center: Vec3,
    ) {
        let distance = size1.length() + size2.length() + 1.0;
        let delta = distance * direction;
        let b1 = Aabb::new(center, size1);
        let b2 = Aabb::new(center + delta, size2);
        assert!(!b1.intersect_aabb(b2));
    }
}
