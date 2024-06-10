use bytemuck::{Pod, Zeroable};
use glam::{Mat2, Mat3, Vec2, Vec3, Vec4};
use itertools::multizip;

use super::resources::VertexAttributeLayout;

pub trait Meshable {
    fn mesh(&self) -> Mesh;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec4,
}
impl VertexAttributeLayout for Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
            wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x2,3=>Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

impl Vertex {
    #[must_use]
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2, tangent: Vec4) -> Self {
        Self {
            position,
            normal,
            uv,
            tangent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub indices: Vec<usize>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub tangents: Vec<Vec4>,
}

impl Mesh {
    pub fn new(
        positions: Vec<Vec3>,
        indices: Vec<usize>,
        normals: Vec<Vec3>,
        uvs: Vec<Vec2>,
    ) -> Self {
        let mut mesh = Self {
            positions,
            indices,
            normals,
            uvs,
            tangents: vec![],
        };

        mesh.recalculate_tangents();
        mesh
    }
    pub fn recalculate_tangents(&mut self) {
        let vertex_count = self.positions.len();
        self.tangents = vec![Vec4::ZERO; vertex_count];
        let mut tangents = vec![Vec3::ZERO; vertex_count];
        let mut bitangents = vec![Vec3::ZERO; vertex_count];
        self.indices.chunks_exact(3).for_each(|t| {
            let i1 = t[0];
            let i2 = t[1];
            let i3 = t[2];
            // Absolute positions
            let v1 = self.positions[i1];
            let v2 = self.positions[i2];
            let v3 = self.positions[i3];
            // Absolute uvs
            let uv1 = self.uvs[i1];
            let uv2 = self.uvs[i2];
            let uv3 = self.uvs[i3];
            // Relative positions
            let q1 = v2 - v1;
            let q2 = v3 - v1;
            // Relative uvs
            let st1 = uv2 - uv1;
            let st2 = uv3 - uv1;
            // Calculate tangent and bitangent
            let st_matrix = Mat3::from_mat2(Mat2::from_cols(st1, st2).transpose());
            let q_matrix = Mat3::from_cols(q1, q2, Vec3::Z).transpose();
            let tangent_bitangent = (st_matrix.inverse() * q_matrix).transpose();
            let tangent = tangent_bitangent.x_axis;
            let bitangent = tangent_bitangent.y_axis;

            tangents[i1] = tangent;
            tangents[i2] = tangent;
            tangents[i3] = tangent;

            bitangents[i1] = bitangent;
            bitangents[i2] = bitangent;
            bitangents[i3] = bitangent;
        });
        for (i, tangent_bitangent) in self.tangents.iter_mut().enumerate() {
            if i == 0 || i == vertex_count - 1 {
                continue;
            }
            fn orthonormalize(matrix: Mat3) -> Mat3 {
                let v1 = matrix.x_axis;
                let v2 = matrix.y_axis.reject_from(v1);
                let v3 = matrix.z_axis.reject_from(v1).reject_from(v2);
                Mat3::from_cols(v1, v2, v3)
            }
            let normal = self.normals[i];
            let tangent = tangents[i];
            let bitangent = bitangents[i];
            let tangent_space =
                orthonormalize(Mat3::from_cols(tangent, bitangent, normal)).transpose();
            let tangent = tangent_space.row(0);
            let bitangent_sign = tangent_space.determinant();
            *tangent_bitangent = tangent.extend(bitangent_sign);
        }
    }

    pub fn recalculate_normals(&mut self) {
        self.normals = vec![Vec3::ZERO; self.positions.len()];
        self.indices.chunks_exact(3).for_each(|t| {
            let v1: Vec3 = self.positions[t[0]];
            let v2: Vec3 = self.positions[t[1]];
            let v3: Vec3 = self.positions[t[2]];
            // Unnormalized, bigger areas contribute more to the normals
            self.normals[t[0]] += (v2 - v1).cross(v3 - v1);
            self.normals[t[1]] += (v2 - v1).cross(v3 - v1);
            self.normals[t[2]] += (v2 - v1).cross(v3 - v1);
        });
        for normal in &mut self.normals {
            *normal = normal.normalize_or_zero();
        }
    }
    #[must_use]
    pub fn vertices(&self) -> Vec<Vertex> {
        let n_vertices = self.positions.len();
        assert_eq!(n_vertices, self.normals.len());
        multizip((&self.positions, &self.normals, &self.uvs, &self.tangents))
            .map(|(p, n, uv, t)| Vertex::new(*p, *n, *uv, *t))
            .collect()
    }
    #[must_use]
    pub fn indices(&self) -> Vec<u32> {
        self.indices.iter().map(|i| *i as u32).collect()
    }
}
