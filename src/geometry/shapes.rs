use approx::{abs_diff_eq, assert_abs_diff_eq};
use glam::{Mat2, Vec2, Vec3, Vec3Swizzles};

use crate::renderer::mesh::{Mesh, Meshable};

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub v1: Vec3,
    pub v2: Vec3,
    pub v3: Vec3,
}

impl Triangle {
    #[must_use]
    pub const fn new(v1: Vec3, v2: Vec3, v3: Vec3) -> Self {
        Self { v1, v2, v3 }
    }
    #[must_use]
    pub fn normal(&self) -> Vec3 {
        (self.v2 - self.v1).cross(self.v3 - self.v1)
    }
    #[must_use]
    pub fn is_inside_triangle(&self, point: Vec3) -> bool {
        // Calculate baricentric coordinates to check if it is inside the triangle
        let r = point - self.v1;
        let q1 = self.v2 - self.v1;
        let q2 = self.v3 - self.v1;
        let dot = q1.dot(q2);
        let coefficients = Mat2::from_cols(
            [q1.length_squared(), dot].into(),
            [dot, q2.length_squared()].into(),
        );
        let constants = Vec2::new(r.dot(q1), r.dot(q2));
        let weights = coefficients.inverse() * constants;
        weights.x >= 0.0 && weights.y >= 0.0 && weights.x + weights.y <= 1.0
    }
}
impl Meshable for Triangle {
    fn mesh(&self) -> Mesh {
        let vertices = vec![self.v1, self.v2, self.v3];
        let triangles = vec![0, 1, 2];
        let normal = self.normal();
        let normals = vec![normal, normal, normal];
        Mesh::new(vertices, triangles, normals)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cuboid {
    pub size: Vec3,
}

impl Cuboid {
    #[must_use]
    pub const fn new(size: Vec3) -> Self {
        Self { size }
    }
    #[must_use]
    pub fn is_point_on_surface(&self, point: Vec3) -> bool {
        abs_diff_eq!(point.x, 0.0, epsilon = 1e-2)
            || abs_diff_eq!(point.x, self.size.x, epsilon = 1e-2)
            || abs_diff_eq!(point.y, 0.0, epsilon = 1e-2)
            || abs_diff_eq!(point.y, self.size.y, epsilon = 1e-2)
            || abs_diff_eq!(point.z, 0.0, epsilon = 1e-2)
            || abs_diff_eq!(point.z, self.size.z, epsilon = 1e-2)
    }
    #[must_use]
    pub fn is_point_inside(&self, point: Vec3) -> bool {
        point.x >= 0.0 && point.x <= self.size.x
            || point.y >= 0.0 && point.y <= self.size.y
            || point.z >= 0.0 && point.z <= self.size.z
    }
}

impl Meshable for Cuboid {
    fn mesh(&self) -> Mesh {
        let max = self.size * 0.5;
        let min = -self.size * 0.5;
        // Suppose Y-up right hand, and camera look from +Z to -Z
        let vertices = [
            // Front
            ([min.x, min.y, max.z], [0.0, 0.0, 1.0]),
            ([max.x, min.y, max.z], [0.0, 0.0, 1.0]),
            ([max.x, max.y, max.z], [0.0, 0.0, 1.0]),
            ([min.x, max.y, max.z], [0.0, 0.0, 1.0]),
            // Back
            ([min.x, max.y, min.z], [0.0, 0.0, -1.0]),
            ([max.x, max.y, min.z], [0.0, 0.0, -1.0]),
            ([max.x, min.y, min.z], [0.0, 0.0, -1.0]),
            ([min.x, min.y, min.z], [0.0, 0.0, -1.0]),
            // Right
            ([max.x, min.y, min.z], [1.0, 0.0, 0.0]),
            ([max.x, max.y, min.z], [1.0, 0.0, 0.0]),
            ([max.x, max.y, max.z], [1.0, 0.0, 0.0]),
            ([max.x, min.y, max.z], [1.0, 0.0, 0.0]),
            // Left
            ([min.x, min.y, max.z], [-1.0, 0.0, 0.0]),
            ([min.x, max.y, max.z], [-1.0, 0.0, 0.0]),
            ([min.x, max.y, min.z], [-1.0, 0.0, 0.0]),
            ([min.x, min.y, min.z], [-1.0, 0.0, 0.0]),
            // Top
            ([max.x, max.y, min.z], [0.0, 1.0, 0.0]),
            ([min.x, max.y, min.z], [0.0, 1.0, 0.0]),
            ([min.x, max.y, max.z], [0.0, 1.0, 0.0]),
            ([max.x, max.y, max.z], [0.0, 1.0, 0.0]),
            // Bottom
            ([max.x, min.y, max.z], [0.0, -1.0, 0.0]),
            ([min.x, min.y, max.z], [0.0, -1.0, 0.0]),
            ([min.x, min.y, min.z], [0.0, -1.0, 0.0]),
            ([max.x, min.y, min.z], [0.0, -1.0, 0.0]),
        ];

        let positions: Vec<_> = vertices.iter().map(|(p, _)| (*p).into()).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n)| (*n).into()).collect();

        let indices = vec![
            0, 1, 2, 2, 3, 0, // front
            4, 5, 6, 6, 7, 4, // back
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // top
            20, 21, 22, 22, 23, 20, // bottom
        ];
        Mesh::new(positions, indices, normals)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub radius: f32,
}

impl Sphere {
    #[must_use]
    pub const fn new(radius: f32) -> Self {
        Self { radius }
    }
    #[must_use]
    // Point must be on surface
    pub fn normal(&self, point: Vec3) -> Vec3 {
        assert_abs_diff_eq!(self.equation(point), 0.0, epsilon = 1e-1);
        Self::gradient(point)
    }
    #[must_use]
    pub(crate) fn equation(self, p: Vec3) -> f32 {
        self.radius.mul_add(-self.radius, p.dot(p))
    }
    #[must_use]
    pub(crate) fn gradient(p: Vec3) -> Vec3 {
        2.0 * p * p
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ellipsoid {
    pub radius: Vec3,
}

impl Ellipsoid {
    #[must_use]
    pub const fn new(radius: Vec3) -> Self {
        Self { radius }
    }
    #[must_use]
    pub(crate) fn equation(self, p: Vec3) -> f32 {
        (p * p).dot((self.radius * self.radius).recip()) - 1.0
    }
    #[must_use]
    pub(crate) fn gradient(&self, p: Vec3) -> Vec3 {
        2.0 * p * (self.radius * self.radius).recip()
    }
    #[must_use]
    // Assuming point is on surface
    pub fn normal(&self, point: Vec3) -> Vec3 {
        assert_abs_diff_eq!(self.equation(point), 0.0, epsilon = 1e-1);
        Self::gradient(self, point)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cylinder {
    pub radius_x: f32,
    pub radius_y: f32,
    pub height: f32,
}

impl Cylinder {
    #[must_use]
    pub const fn new(radius_x: f32, radius_y: f32, height: f32) -> Self {
        Self {
            radius_x,
            radius_y,
            height,
        }
    }
    #[must_use]
    pub(crate) fn equation(self, p: Vec3) -> f32 {
        let p_xy = p.xy();
        let radius_xy = Vec2::new(self.radius_x, self.radius_y);
        (p_xy * p_xy).dot((radius_xy * radius_xy).recip()) - 1.0
    }
    #[must_use]
    pub(crate) fn gradient(&self, p: Vec3) -> Vec3 {
        let p_xy = p.xy();
        let radius_xy = Vec2::new(self.radius_x, self.radius_y);
        let v_xy = 2.0 * (p_xy * radius_xy.recip());
        v_xy.extend(if p.z <= 0.0 {
            -1.0
        } else if p.z >= self.height {
            1.0
        } else {
            0.0
        })
    }
    #[must_use]
    // Assuming point is on surface
    pub fn normal(&self, point: Vec3) -> Vec3 {
        assert_abs_diff_eq!(self.equation(point), 0.0, epsilon = 1e-1);
        Self::gradient(self, point)
    }
}
