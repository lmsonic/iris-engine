use std::{
    f32::consts::PI,
    f32::consts::{FRAC_PI_2, TAU},
};

use glam::Vec2;
use glam::Vec3;
use hexasphere::shapes::IcoSphere;

use crate::renderer::mesh::{Mesh, Meshable, Vertex};

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    #[inline]
    pub const fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
    #[inline]
    pub fn contains(&self, point: Vec3) -> bool {
        self.center.distance_squared(point) < self.radius * self.radius
    }
    #[inline]
    pub fn closest_on_sphere(&self, point: Vec3) -> Vec3 {
        let direction = (point - self.center).normalize();
        direction * self.radius + self.center
    }

    pub fn ico(&self, subdivisions: usize) -> Mesh {
        let generated = IcoSphere::new(subdivisions, |point| {
            let inclination = point.y.acos();
            let azimuth = point.z.atan2(point.x);

            let norm_inclination = inclination / PI;
            let norm_azimuth = 0.5 - (azimuth / TAU);

            [norm_azimuth, norm_inclination]
        });

        let raw_points = generated.raw_points();

        let vertices = raw_points
            .iter()
            .zip(generated.raw_data())
            .map(|(pn, uv)| {
                let position = Vec3::new(pn.x, pn.y, pn.z) * self.radius;
                let normal = Vec3::new(pn.x, pn.y, pn.z);
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

        for i in 0..=stacks {
            let stack_angle = (i as f32).mul_add(-stack_step, FRAC_PI_2);
            let xy = self.radius * stack_angle.cos();
            let y = self.radius * stack_angle.sin();

            for j in 0..=sectors {
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
        Self::uv(self, 18, 20)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use glam::Vec3;
    use proptest::proptest;

    use crate::tests::{any_normal, any_vec3};

    use super::Sphere;
    const RANGE: std::ops::RangeInclusive<f32> = -100.0..=100.0;
    proptest! {
        #[test]
        fn contains(center in any_vec3(RANGE),radius in 0.1..=100.0_f32,direction in any_normal(),factor in 0.01..=0.99_f32){
            let point = direction * factor * radius + center;
            _contains(center, radius, point);
        }
        #[test]
        fn not_contains(center in any_vec3(RANGE),radius in 0.1..=100.0_f32,direction in any_normal(),factor in 1.1..=100.0_f32){
            let point = direction * factor * radius + center;
            _not_contains(center, radius, point);
        }
        #[test]
        fn closest_on_sphere(center in any_vec3(RANGE),radius in 0.1..=100.0_f32,direction in any_normal(),factor in 1.1..=100.0_f32){
            let point = direction * factor * radius+ center;
            let expected = direction * radius+ center;
            _closest_on_sphere(center, radius, point,expected);
        }
    }
    fn _contains(center: Vec3, radius: f32, point: Vec3) {
        let sphere = Sphere::new(center, radius);
        assert!(sphere.contains(point));
    }
    fn _not_contains(center: Vec3, radius: f32, point: Vec3) {
        let sphere = Sphere::new(center, radius);
        assert!(!sphere.contains(point));
    }
    fn _closest_on_sphere(center: Vec3, radius: f32, point: Vec3, expected: Vec3) {
        let sphere = Sphere::new(center, radius);
        assert_abs_diff_eq!(sphere.closest_on_sphere(point), expected, epsilon = 1e-3);
    }
}
