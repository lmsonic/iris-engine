use bytemuck::{Pod, Zeroable};

use glam::{Mat4, Vec3, Vec4};

use super::color::Color;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuLight {
    position: Vec4,
    color_range: Vec4,
    custom_data: Vec4,
}

pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
}

impl DirectionalLight {
    #[must_use]
    pub fn new(color: Color, direction: Vec3) -> Self {
        Self {
            direction: direction.normalize(),
            color: color.into(),
        }
    }
    pub fn to_gpu(&self) -> GpuLight {
        GpuLight {
            position: -self.direction.extend(0.0),
            color_range: self.color.extend(0.0),
            custom_data: Default::default(),
        }
    }
}

pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub range: f32,
    pub attenuation: [f32; 3],
}

impl PointLight {
    #[must_use]
    pub fn new(color: Color, position: Vec3) -> Self {
        Self {
            position,
            color: color.into(),
            range: 100.0,
            attenuation: [0.0, 2.0, 0.0],
        }
    }
    pub fn to_gpu(&self) -> GpuLight {
        GpuLight {
            position: self.position.extend(1.0),
            color_range: self.color.extend(self.range),
            custom_data: Vec4::new(
                self.attenuation[0],
                self.attenuation[1],
                self.attenuation[2],
                // To flag its a point light
                -1.0,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpotLight {
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub range: f32,
    pub outer_cutoff: f32,
}

impl SpotLight {
    #[must_use]
    pub fn new(
        color: Color,
        position: Vec3,
        direction: Vec3,
        range: f32,
        outer_cutoff: f32,
    ) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            color: color.into(),
            range,
            outer_cutoff,
        }
    }

    pub fn to_gpu(&self) -> GpuLight {
        GpuLight {
            position: self.position.extend(1.0),
            color_range: self.color.extend(self.range),
            custom_data: self.direction.extend(self.outer_cutoff.cos()),
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
