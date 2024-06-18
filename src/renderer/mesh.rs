use std::{fmt::Debug, path::Path};

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};

use crate::visibility::bounding_volume::aabb::Aabb;

use super::resources::{load_geometry, VertexAttributeLayout};

pub trait Meshable {
    fn mesh(&self) -> Mesh;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, Default, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}
impl VertexAttributeLayout for Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x2,3=>Float32x3,4=>Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

impl Vertex {
    pub const fn new(
        position: Vec3,
        normal: Vec3,
        uv: Vec2,
        tangent: Vec3,
        bitangent: Vec3,
    ) -> Self {
        Self {
            position,
            normal,
            uv,
            tangent,
            bitangent,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn compute_tangent_frame(face: [Vertex; 3], expected_normal: Vec3) -> (Vec3, Vec3) {
    let e1_pos = face[1].position - face[0].position;
    let e2_pos = face[2].position - face[0].position;

    let e1_uv = face[1].uv - face[0].uv;
    let e2_uv = face[2].uv - face[0].uv;

    let mut tangent = (e1_pos * e2_uv.y - e2_pos * e1_uv.y).normalize();
    let mut bitangent = (e2_pos * e1_uv.x - e1_pos * e2_uv.x).normalize();
    let mut normal = tangent.cross(bitangent);

    if normal.dot(expected_normal) < 0.0 {
        tangent = -tangent;
    }

    normal = expected_normal;
    tangent = (tangent - tangent.dot(normal) * normal).normalize();
    bitangent = normal.cross(tangent);

    (tangent, bitangent)
}

impl Mesh {
    pub fn from_obj(path: impl AsRef<Path> + Debug) -> Self {
        load_geometry(path)
    }
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let mut mesh = Self { vertices, indices };

        mesh.recalculate_tangents();
        mesh
    }
    pub fn recalculate_tangents(&mut self) {
        for i in self.indices.chunks_exact(3) {
            let v1 = self.vertices[i[0] as usize];
            let v2 = self.vertices[i[1] as usize];
            let v3 = self.vertices[i[2] as usize];
            for j in i.iter().take(3) {
                let v = &mut self.vertices[*j as usize];
                let (tangent, bitangent) = compute_tangent_frame([v1, v2, v3], v.normal);
                v.tangent = tangent;
                v.bitangent = bitangent;
            }
        }
    }

    pub fn recalculate_normals(&mut self) {
        self.indices.chunks_exact(3).for_each(|t| {
            let v1 = self.vertices[t[0] as usize].position;
            let v2 = self.vertices[t[1] as usize].position;
            let v3 = self.vertices[t[2] as usize].position;
            // Unnormalized, bigger areas contribute more to the normals
            self.vertices[t[0] as usize].normal += (v2 - v1).cross(v3 - v1);
            self.vertices[t[1] as usize].normal += (v2 - v1).cross(v3 - v1);
            self.vertices[t[2] as usize].normal += (v2 - v1).cross(v3 - v1);
        });
        for v in &mut self.vertices {
            v.normal = v.normal.normalize_or_zero();
        }
    }

    pub fn calculate_bounding_box(&self) -> Aabb {
        let points: Vec<Vec3> = self.vertices.iter().map(|v| v.position).collect();
        Aabb::from_points(&points)
    }

    pub fn vertices(&self) -> Vec<Vertex> {
        self.vertices.clone()
    }

    pub fn indices(&self) -> Vec<u32> {
        self.indices.clone()
    }
}
