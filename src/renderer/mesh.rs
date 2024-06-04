use encase::ShaderType;
use glam::{Vec3, Vec4};

use super::color::Color;

#[repr(C)]
#[derive(Clone, Copy, Debug, ShaderType)]
pub struct Vertex {
    pub position: Vec4,
    pub normal: Vec3,
    pub color: Color,
}

impl Vertex {
    #[must_use]
    pub fn new(position: Vec3, normal: Vec3, color: Color) -> Self {
        Self {
            position: position.extend(1.0),
            normal,
            color,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct IndexTriangle {
    pub v1: usize,
    pub v2: usize,
    pub v3: usize,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<IndexTriangle>,
    pub normals: Vec<Vec3>,
    pub colors: Vec<Color>,
}

impl Mesh {
    pub fn recalculate_normals(&mut self) {
        for (index, _) in self.vertices.iter().enumerate() {
            let normal: Vec3 = self
                .triangles
                .iter()
                .filter(|t| t.v1 == index || t.v2 == index || t.v3 == index)
                .map(|t| {
                    let v1 = self.vertices[t.v1];
                    let v2 = self.vertices[t.v2];
                    let v3 = self.vertices[t.v3];
                    // Unnormalized, bigger areas contribute more to the normals
                    (v2 - v1).cross(v3 - v1)
                })
                .sum();
            self.normals[index] = normal.normalize();
        }
    }
    #[must_use]
    pub fn vertices(&self) -> Vec<Vertex> {
        let n_vertices = self.vertices.len();
        assert_eq!(n_vertices, self.normals.len());
        assert_eq!(n_vertices, self.colors.len());
        assert_eq!(n_vertices, self.triangles.len() / 3);

        self.vertices
            .iter()
            .zip(self.normals.iter())
            .zip(self.colors.iter())
            .map(|((position, normal), color)| Vertex::new(*position, *normal, *color))
            .collect()
    }
    #[must_use]
    pub fn indices(&self) -> Vec<usize> {
        self.triangles
            .iter()
            .flat_map(|v| [v.v1, v.v2, v.v3])
            .collect()
    }
}
