use glam::{Mat2, Vec2, Vec3};

use crate::{
    collision::linear_systems::solve_linear_system_2d,
    renderer::mesh::{Mesh, Meshable, Vertex},
};

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
        (self.v2 - self.v1).cross(self.v3 - self.v1).normalize()
    }

    pub fn baricentric_coordinates(&self, point: Vec3) -> (f32, f32, f32) {
        let d0 = self.v2 - self.v1;
        let d1 = self.v3 - self.v1;
        let d2 = point - self.v1;
        let dot = d0.dot(d1);
        let coefficients = Mat2::from_cols(
            [d0.length_squared(), dot].into(),
            [dot, d1.length_squared()].into(),
        );
        let constants = Vec2::new(d2.dot(d0), d2.dot(d1));
        let baricentric = solve_linear_system_2d(coefficients, constants)
            .expect("Tried to calculate baricentric coordinates of degenerate triangle");
        let v = baricentric.x;
        let w = baricentric.y;
        (1.0 - v - w, v, w)
    }

    pub(crate) fn contains(&self, point: Vec3) -> bool {
        let (_, v, w) = self.baricentric_coordinates(point);
        v >= 0.0 && w >= 0.0 && v + w <= 1.0
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
