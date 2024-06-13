use bytemuck::{Pod, Zeroable};

use egui::Ui;
use glam::{Mat4, Vec3, Vec4};

use crate::GpuSendable;

use super::{
    color::Color,
    gui::{array3_edit, color_edit, direction_edit, drag_angle_clamp, float_edit, vec3_edit},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuLight {
    position: Vec4,
    color_range: Vec4,
    custom_data: Vec4,
}

#[derive(Debug, Clone, Copy)]
pub enum Light {
    DirectionalLight(DirectionalLight),
    PointLight(PointLight),
    SpotLight(SpotLight),
}

impl GpuSendable<GpuLight> for Light {
    fn to_gpu(&self) -> GpuLight {
        match self {
            Self::DirectionalLight(light) => light.to_gpu(),
            Self::PointLight(light) => light.to_gpu(),
            Self::SpotLight(light) => light.to_gpu(),
        }
    }
}
impl From<DirectionalLight> for Light {
    fn from(value: DirectionalLight) -> Self {
        Self::DirectionalLight(value)
    }
}
impl From<PointLight> for Light {
    fn from(value: PointLight) -> Self {
        Self::PointLight(value)
    }
}
impl From<SpotLight> for Light {
    fn from(value: SpotLight) -> Self {
        Self::SpotLight(value)
    }
}

impl Light {
    pub fn gui(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;
        match self {
            Light::DirectionalLight(ref mut light) => {
                ui.label("Directional Light");
                changed |= color_edit(ui, &mut light.color, "Color");
                changed |= direction_edit(ui, &mut light.direction, "Direction");
            }
            Light::PointLight(ref mut light) => {
                ui.label("Point Light");
                changed |= color_edit(ui, &mut light.color, "Color");
                changed |= vec3_edit(ui, &mut light.position, "Position", -10.0..=10.0);
                changed |= float_edit(ui, &mut light.range, "Range", 1.0..=100.0);
                changed |= array3_edit(
                    ui,
                    &mut light.attenuation,
                    "Attenuation function",
                    0.0..=5.0,
                );
            }
            Light::SpotLight(ref mut light) => {
                ui.label("Spot Light");
                changed |= color_edit(ui, &mut light.color, "Color");
                changed |= direction_edit(ui, &mut light.direction, "Direction");
                changed |= vec3_edit(ui, &mut light.position, "Position", -10.0..=10.0);
                changed |= float_edit(ui, &mut light.range, "Range", 1.0..=100.0);
                ui.horizontal(|ui| {
                    changed |= drag_angle_clamp(ui, &mut light.outer_cutoff, 0.0..=90.0).changed();
                    light.outer_cutoff = f32::max(0.0, light.outer_cutoff);
                    ui.label("Cutoff");
                });
            }
        };
        changed
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::NEG_ONE,
            color: Color::WHITE.into(),
        }
    }
}

impl DirectionalLight {
    #[must_use]
    pub fn new(color: Color, direction: Vec3) -> Self {
        Self {
            color: color.into(),
            direction: direction.normalize(),
        }
    }
}

impl GpuSendable<GpuLight> for DirectionalLight {
    fn to_gpu(&self) -> GpuLight {
        GpuLight {
            position: -self.direction.extend(0.0),
            color_range: self.color.extend(0.0),
            custom_data: Default::default(),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub range: f32,
    pub attenuation: [f32; 3],
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Color::WHITE.into(),
            position: Vec3::ZERO,
            range: 100.0,
            attenuation: [0.0, 0.0, 1.0],
        }
    }
}

impl PointLight {
    #[must_use]
    pub fn new(color: Color, position: Vec3, range: f32, attenuation: [f32; 3]) -> Self {
        Self {
            position,
            color: color.into(),
            range,
            attenuation,
        }
    }
}

impl GpuSendable<GpuLight> for PointLight {
    fn to_gpu(&self) -> GpuLight {
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

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            direction: Vec3::Z,
            color: Color::WHITE.into(),
            range: 100.0,
            outer_cutoff: f32::to_radians(30.0),
        }
    }
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

impl GpuSendable<GpuLight> for SpotLight {
    fn to_gpu(&self) -> GpuLight {
        GpuLight {
            position: self.position.extend(1.0),
            color_range: self.color.extend(self.range),
            custom_data: self.direction.extend(self.outer_cutoff.cos()),
        }
    }
}
