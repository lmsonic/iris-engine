use std::f32::consts::PI;

use approx::{abs_diff_eq, assert_abs_diff_eq};
use glam::{Mat2, Vec2, Vec3, Vec3Swizzles};
use hexasphere::shapes::IcoSphere;

use crate::renderer::mesh::{Mesh, Meshable, Vertex};

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
        let min = -self.size * 0.5;
        let max = self.size * 0.5;

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
            .map(|(p, n, u)| Vertex {
                position: Vec3::from(*p),
                normal: Vec3::from(*n),
                uv: Vec2::from(*u),
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
    pub fn ico(&self, subdivisions: usize) -> Mesh {
        let generated = IcoSphere::new(subdivisions, |point| {
            let inclination = point.y.acos();
            let azimuth = point.z.atan2(point.x);

            let norm_inclination = inclination / std::f32::consts::PI;
            let norm_azimuth = 0.5 - (azimuth / std::f32::consts::TAU);

            [norm_azimuth, norm_inclination]
        });

        let raw_points = generated.raw_points();

        let vertices = raw_points
            .iter()
            .zip(generated.raw_data())
            .map(|(pn, uv)| {
                let position: Vec3 = Vec3::from(*pn) * self.radius;
                let normal: Vec3 = Vec3::from(*pn);
                let uv: Vec2 = Vec2::from(*uv);
                Vertex {
                    position,
                    normal,
                    uv,
                    ..Default::default()
                }
            })
            .collect();

        let mut indices = Vec::with_capacity(generated.indices_per_main_triangle() * 20);

        for i in 0..20 {
            generated.get_indices(i, &mut indices);
        }

        let indices = indices.into_iter().collect();
        Mesh::new(vertices, indices)
    }
    pub fn uv(&self, sectors: usize, stacks: usize) -> Mesh {
        // From https://docs.rs/bevy_render/latest/src/bevy_render/mesh/primitives/dim3/sphere.rs.html#182

        // Largely inspired from http://www.songho.ca/opengl/gl_sphere.html

        let sectors_f32 = sectors as f32;
        let stacks_f32 = stacks as f32;
        let length_inv = 1. / self.radius;
        let sector_step = 2. * PI / sectors_f32;
        let stack_step = PI / stacks_f32;

        let mut vertices = Vec::with_capacity(stacks * sectors);

        let mut indices = Vec::with_capacity(stacks * sectors * 2 * 3);

        for i in 0..stacks + 1 {
            let stack_angle = PI / 2. - (i as f32) * stack_step;
            let xy = self.radius * stack_angle.cos();
            let y = self.radius * stack_angle.sin();

            for j in 0..sectors + 1 {
                let sector_angle = (j as f32) * sector_step;
                let x = xy * sector_angle.cos();
                let z = xy * sector_angle.sin();
                vertices.push(Vertex {
                    position: [x, y, -z].into(),
                    normal: [x * length_inv, y * length_inv, -z * length_inv].into(),
                    uv: [(j as f32) / sectors_f32, (i as f32) / stacks_f32].into(),
                    ..Default::default()
                });
            }
        }

        // indices
        //  k1--k1+1
        //  |  / |
        //  | /  |
        //  k2--k2+1
        for i in 0..stacks {
            let mut k1 = i * (sectors + 1);
            let mut k2 = k1 + sectors + 1;
            for _j in 0..sectors {
                if i != 0 {
                    indices.push(k1 as u32);
                    indices.push(k2 as u32);
                    indices.push((k1 + 1) as u32);
                }
                if i != stacks - 1 {
                    indices.push((k1 + 1) as u32);
                    indices.push(k2 as u32);
                    indices.push((k2 + 1) as u32);
                }
                k1 += 1;
                k2 += 1;
            }
        }

        Mesh::new(vertices, indices)
    }
}
impl Meshable for Sphere {
    fn mesh(&self) -> Mesh {
        Sphere::uv(self, 18, 20)
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
