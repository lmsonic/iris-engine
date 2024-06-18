use glam::{Mat2, Vec2, Vec3};

use crate::renderer::mesh::{Mesh, Meshable, Vertex};

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub v1: Vec3,
    pub v2: Vec3,
    pub v3: Vec3,
}

impl Triangle {
    pub const fn new(v1: Vec3, v2: Vec3, v3: Vec3) -> Self {
        Self { v1, v2, v3 }
    }

    pub fn normal(&self) -> Vec3 {
        (self.v2 - self.v1).cross(self.v3 - self.v1)
    }

    pub(crate) fn is_inside_triangle(&self, point: Vec3) -> bool {
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
        let normal = self.normal();

        let v1 = Vertex {
            position: self.v1,
            normal,
            uv: [0.0, 0.0].into(),
            ..Default::default()
        };
        let v2 = Vertex {
            position: self.v2,
            normal,
            uv: [0.0, 1.0].into(),
            ..Default::default()
        };
        let v3 = Vertex {
            position: self.v3,
            normal,
            uv: [1.0, 0.0].into(),
            ..Default::default()
        };
        let vertices = vec![v1, v2, v3];
        let indices = vec![0, 1, 2];
        Mesh::new(vertices, indices)
    }
}
