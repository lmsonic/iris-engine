use glam::Vec3;

use crate::color::Color;

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
}
