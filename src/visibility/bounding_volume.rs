use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use glam::{Affine3A, Quat, Vec2, Vec3};

use crate::renderer::mesh::{Mesh, Meshable, Vertex};

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
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
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}
impl Add<Aabb> for Vec3 {
    type Output = Aabb;

    fn add(self, rhs: Aabb) -> Self::Output {
        Self::Output {
            min: self + rhs.min,
            max: self + rhs.max,
        }
    }
}

impl Sub<Vec3> for Aabb {
    type Output = Self;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            min: self.min - rhs,
            max: self.max - rhs,
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
            min: self.min * rhs,
            max: self.max * rhs,
        }
    }
}
impl Mul<Aabb> for Vec3 {
    type Output = Aabb;

    fn mul(self, rhs: Aabb) -> Self::Output {
        Self::Output {
            min: self * rhs.min,
            max: self * rhs.max,
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
            min: self.min * rhs,
            max: self.max * rhs,
        }
    }
}
impl Mul<Aabb> for f32 {
    type Output = Aabb;

    fn mul(self, rhs: Aabb) -> Self::Output {
        Self::Output {
            min: self * rhs.min,
            max: self * rhs.max,
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
                min: scale * rhs.min + translation,
                max: scale * rhs.max + translation,
            },
        }
    }
}

impl Aabb {
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            || point.y >= self.min.y && point.y <= self.max.y
            || point.z >= self.min.z && point.z <= self.max.z
    }
    pub fn from_points(points: &[Vec3]) -> Self {
        // Transform all points by rotation
        let first = points
            .iter()
            .next()
            .expect("mesh must contain at least one point for Aabb construction");

        let (min, max) = points
            .iter()
            .copied()
            .fold((*first, *first), |(prev_min, prev_max), point| {
                (point.min(prev_min), point.max(prev_max))
            });

        Self { min, max }
    }
}
impl Meshable for Aabb {
    fn mesh(&self) -> Mesh {
        let min = self.min;
        let max = self.max;

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

#[derive(Debug, Clone, Copy)]
pub struct Obb {
    // Rotation gets applied in the local space of Aabb
    pub rotation: Quat,
    pub aabb: Aabb,
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

impl Obb {
    pub const fn new(rotation: Quat, min: Vec3, max: Vec3) -> Self {
        Self {
            rotation,
            aabb: Aabb { min, max },
        }
    }
    pub fn center(&self) -> Vec3 {
        self.aabb.center()
    }

    pub fn contains(&self, point: Vec3) -> bool {
        let center = self.center();

        // Transform point to the obb space
        let point = self.rotation.inverse() * (point - center);

        self.aabb.contains(point)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

// Translation operations

impl AddAssign<Vec3> for BoundingSphere {
    fn add_assign(&mut self, rhs: Vec3) {
        *self = *self + rhs;
    }
}
impl SubAssign<Vec3> for BoundingSphere {
    fn sub_assign(&mut self, rhs: Vec3) {
        *self = *self - rhs;
    }
}

impl Add<Vec3> for BoundingSphere {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            center: self.center + rhs,
            radius: self.radius,
        }
    }
}
impl Add<BoundingSphere> for Vec3 {
    type Output = BoundingSphere;

    fn add(self, rhs: BoundingSphere) -> Self::Output {
        Self::Output {
            center: rhs.center + self,
            radius: rhs.radius,
        }
    }
}

impl Sub<Vec3> for BoundingSphere {
    type Output = Self;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            center: self.center - rhs,
            radius: self.radius,
        }
    }
}

// Scale operations

impl MulAssign<f32> for BoundingSphere {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Mul<f32> for BoundingSphere {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            center: self.center,
            radius: self.radius * rhs,
        }
    }
}
impl Mul<BoundingSphere> for f32 {
    type Output = BoundingSphere;

    fn mul(self, rhs: BoundingSphere) -> Self::Output {
        Self::Output {
            center: rhs.center,
            radius: rhs.radius * self,
        }
    }
}

impl BoundingSphere {
    pub fn from_points(points: &[Vec3]) -> Self {
        let center = center_of_points(points);
        let mut radius_squared = 0.0;

        for point in points {
            // Get squared version to avoid unnecessary sqrt calls
            let distance_squared = point.distance_squared(center);
            if distance_squared > radius_squared {
                radius_squared = distance_squared;
            }
        }

        Self {
            center,
            radius: radius_squared.sqrt(),
        }
    }
    pub fn contains(&self, point: Vec3) -> bool {
        self.center.distance_squared(point) < self.radius * self.radius
    }
}

pub fn center_of_points(points: &[Vec3]) -> Vec3 {
    assert!(
        !points.is_empty(),
        "cannot compute the center of an empty mesh"
    );

    let denom = 1.0 / points.len() as f32;
    points.iter().fold(Vec3::ZERO, |acc, point| acc + *point) * denom
}
