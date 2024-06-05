use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

pub trait Meshable {
    fn mesh(&self) -> Mesh;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: Vec4,
    pub normal: Vec3,
    _padding: f32,
}

impl Vertex {
    #[must_use]
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Self {
            position: position.extend(1.0),
            normal,
            _padding: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub indices: Vec<usize>,
    pub normals: Vec<Vec3>,
}

impl Mesh {
    pub fn new(positions: Vec<Vec3>, indices: Vec<usize>, normals: Vec<Vec3>) -> Self {
        Self {
            positions,
            indices,
            normals,
        }
    }

    pub fn recalculate_normals(&mut self) {
        for (index, _) in self.positions.iter().enumerate() {
            let normal: Vec3 = self
                .indices
                .chunks_exact(3)
                .filter(|t| t[0] == index || t[1] == index || t[2] == index)
                .map(|t| {
                    let v1: Vec3 = self.positions[t[0]];
                    let v2: Vec3 = self.positions[t[1]];
                    let v3: Vec3 = self.positions[t[2]];
                    // Unnormalized, bigger areas contribute more to the normals
                    (v2 - v1).cross(v3 - v1)
                })
                .sum();
            self.normals[index] = normal.normalize();
            todo!()
        }
    }
    #[must_use]
    pub fn vertices(&self) -> Vec<Vertex> {
        let n_vertices = self.positions.len();
        assert_eq!(n_vertices, self.normals.len());

        self.positions
            .iter()
            .zip(self.normals.iter())
            .map(|(position, normal)| Vertex::new(*position, *normal))
            .collect()
    }
    #[must_use]
    pub fn indices(&self) -> Vec<u32> {
        self.indices.iter().map(|i| *i as u32).collect()
    }
}
