use encase::ShaderType;
use glam::{Mat4, Vec3, Vec4};

use super::color::Color;

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Color,
}

impl DirectionalLight {
    #[must_use]
    pub const fn new(color: Color, direction: Vec3) -> Self {
        Self { direction, color }
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Color,
    pub range: f32,
    pub attenuation: [f32; 3],
}

impl PointLight {
    #[must_use]
    pub const fn new(color: Color, position: Vec3, range: f32, attenuation: [f32; 3]) -> Self {
        Self {
            position,
            color,
            range,
            attenuation,
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct SpotLight {
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Color,
    pub range: f32,
    pub inner_cutoff: f32,
    pub outer_cutoff: f32,
}

impl SpotLight {
    #[must_use]
    pub const fn new(
        position: Vec3,
        direction: Vec3,
        color: Color,
        range: f32,
        inner_cutoff: f32,
        outer_cutoff: f32,
    ) -> Self {
        Self {
            position,
            direction,
            color,
            range,
            inner_cutoff,
            outer_cutoff,
        }
    }

    #[must_use]
    pub fn project_texture_matrix(&self, width: usize, height: usize) -> Mat4 {
        let z_axis = self.direction;
        // Assume texture uses up as T direction
        let y_axis = Vec3::Z;
        let x_axis = y_axis.cross(z_axis);
        let w_axis = Vec4::new(
            -x_axis.dot(self.position),
            -y_axis.dot(self.position),
            -z_axis.dot(self.position),
            1.0,
        );
        let from_world_to_light = Mat4::from_cols(
            x_axis.extend(0.0),
            y_axis.extend(0.0),
            z_axis.extend(0.0),
            w_axis,
        );

        let f = 1.0 / f32::tan(0.5 * self.outer_cutoff);
        let aspect_ratio = width as f32 / height as f32;
        let projection = Mat4::from_cols(
            Vec4::X * f * 0.5,
            Vec4::Y * f * 0.5 * aspect_ratio.recip(),
            Vec4::new(0.5, 0.5, 0.0, 1.0),
            Vec4::ZERO,
        );
        from_world_to_light * projection
    }
}
