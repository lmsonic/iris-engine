use glam::{Mat4, Vec3, Vec4};

use crate::color::Color;

pub trait Light {
    fn intensity(&self, point: Vec3) -> Color;
    fn direction(&self, point: Vec3) -> Vec3;
}

#[derive(Clone, Copy, Debug)]
pub struct DirectionalLight {
    pub color: Color,
    pub direction: Vec3,
}

impl DirectionalLight {
    #[must_use]
    pub const fn new(color: Color, direction: Vec3) -> Self {
        Self { color, direction }
    }
}
impl Light for DirectionalLight {
    fn intensity(&self, _point: Vec3) -> Color {
        self.color
    }

    fn direction(&self, _point: Vec3) -> Vec3 {
        -self.direction
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub color: Color,
    pub center: Vec3,
    pub range: f32,
    pub attenuation: [f32; 3],
}

impl PointLight {
    #[must_use]
    pub const fn new(color: Color, center: Vec3, range: f32, attenuation: [f32; 3]) -> Self {
        Self {
            color,
            center,
            range,
            attenuation,
        }
    }
}
impl Light for PointLight {
    fn intensity(&self, point: Vec3) -> Color {
        let distance = point.distance(self.center);
        if distance > self.range {
            return Color::BLACK;
        }
        let den = (distance * distance).mul_add(
            self.attenuation[2],
            distance.mul_add(self.attenuation[1], self.attenuation[0]),
        );
        self.color * den.recip()
    }

    fn direction(&self, point: Vec3) -> Vec3 {
        self.center - point
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpotLight {
    pub color: Color,
    pub center: Vec3,
    pub direction: Vec3,
    pub range: f32,
    pub angle: f32,
    pub angle_attenuation: f32,
    pub attenuation: [f32; 3],
}

impl SpotLight {
    #[must_use]
    pub const fn new(
        color: Color,
        center: Vec3,
        direction: Vec3,
        range: f32,
        angle: f32,
        angle_attenuation: f32,
        attenuation: [f32; 3],
    ) -> Self {
        Self {
            color,
            center,
            direction,
            range,
            angle,
            angle_attenuation,
            attenuation,
        }
    }
    #[must_use]
    pub fn project_texture_matrix(&self, width: usize, height: usize) -> Mat4 {
        let z_axis = self.direction;
        // Assume texture uses up as T direction
        let y_axis = Vec3::Z;
        let x_axis = y_axis.cross(z_axis);
        let w_axis = Vec4::new(
            -x_axis.dot(self.center),
            -y_axis.dot(self.center),
            -z_axis.dot(self.center),
            1.0,
        );
        let from_world_to_light = Mat4::from_cols(
            x_axis.extend(0.0),
            y_axis.extend(0.0),
            z_axis.extend(0.0),
            w_axis,
        );

        let f = 1.0 / f32::tan(0.5 * self.angle);
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
impl Light for SpotLight {
    fn intensity(&self, point: Vec3) -> Color {
        let delta = (self.center - point).normalize();
        let distance = delta.length();
        let angle = self.direction.angle_between(delta);
        if distance > self.range || angle > self.angle {
            return Color::BLACK;
        }
        let num = f32::max(-self.direction.dot(delta), 0.0).powf(self.angle_attenuation);
        let den = (distance * distance).mul_add(
            self.attenuation[2],
            distance.mul_add(self.attenuation[1], self.attenuation[0]),
        );
        self.color * num * den.recip()
    }

    fn direction(&self, point: Vec3) -> Vec3 {
        self.center - point
    }
}

#[must_use]
pub fn diffuse_reflection(
    point: Vec3,
    normal: Vec3,
    surface_diffuse: Color,
    texture_diffuse: Color,
    ambient: f32,
    lights: &[Box<dyn Light>],
) -> Color {
    let mut lighting = Color::BLACK;
    for light in lights {
        let direction = light.direction(point);
        lighting += light.intensity(point) * normal.dot(direction).max(0.0);
    }
    let diffuse_color = surface_diffuse * texture_diffuse;
    diffuse_color * ambient + diffuse_color * lighting
}

#[must_use]
pub fn specular_reflection(
    point: Vec3,
    normal: Vec3,
    camera_pos: Vec3,
    surface_specular: Color,
    texture_specular: Color,
    specular_exponent: f32,
    lights: &[Box<dyn Light>],
) -> Color {
    let view = camera_pos - point;
    let mut lighting = Color::BLACK;
    for light in lights {
        let light_direction = light.direction(point);
        let half = (light_direction + view).normalize();
        let ndotl = if normal.dot(light_direction) > 0.0 {
            1.0
        } else {
            0.0
        };
        lighting +=
            light.intensity(point) * normal.dot(half).max(0.0).powf(specular_exponent) * ndotl;
    }
    surface_specular * texture_specular * lighting
}

#[must_use]
pub fn emission(emission_color: Color, texture_sample: Color) -> Color {
    emission_color * texture_sample
}
