use egui::load::SizedTexture;
use egui::{Button, DragValue, Response, Slider, TextureId, Ui, Vec2};

use glam::{Affine3A, EulerRot, Quat, Vec3};
use std::f32::consts::PI;
use std::ops::RangeInclusive;

use super::color::Color;
use super::light::{DirectionalLight, Light, PointLight, SpotLight};
use super::material::{LitMaterialBuilder, Material, PbrMaterialBuilder, UnlitMaterialBuilder};

pub fn transform_edit(ui: &mut Ui, transform: &mut Affine3A) -> bool {
    let mut changed = false;
    ui.collapsing("Transform", |ui| {
        let (mut scale, rotation, mut translation) = transform.to_scale_rotation_translation();
        let mut eulers: Vec3 = rotation.to_euler(EulerRot::XYZ).into();
        if vec3_edit(ui, &mut translation, "Translation") {
            transform.translation = translation.into();
            changed = true;
        }

        ui.label("Rotation (Euler)");
        let mut rotation_changed = false;
        ui.horizontal(|ui| {
            ui.label("X");
            rotation_changed |= ui.drag_angle(&mut eulers.x).changed();
        });
        ui.horizontal(|ui| {
            ui.label("Y");
            rotation_changed |= ui.drag_angle(&mut eulers.y).changed();
        });
        ui.horizontal(|ui| {
            ui.label("Z");
            rotation_changed |= ui.drag_angle(&mut eulers.z).changed();
        });
        if rotation_changed {
            let rotation = Quat::from_euler(glam::EulerRot::XYZ, eulers.x, eulers.y, eulers.z);
            *transform = Affine3A::from_scale_rotation_translation(scale, rotation, translation);
            changed = true;
        }
        // TODO: uniform scale toggle
        if vec3_edit(ui, &mut scale, "Scale") && scale.x != 0.0 && scale.y != 0.0 && scale.z != 0.0
        {
            *transform = Affine3A::from_scale_rotation_translation(scale, rotation, translation);
            changed = true;
        }
    });

    changed
}

pub fn texture_edit(ui: &mut Ui, texture_id: TextureId, label: &str) -> bool {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(Button::image(SizedTexture::new(
            texture_id,
            Vec2::splat(100.0),
        )))
    })
    .inner
    .clicked()
}

pub fn change_material(
    ui: &mut Ui,
    material: &mut Box<dyn for<'a> Material<'a>>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> bool {
    let mut changed = false;
    ui.menu_button("Change Material", |ui| {
        if ui.button("Unlit").clicked() {
            *material = Box::new(UnlitMaterialBuilder::new().build(device, queue));
            changed = true;
        }
        if ui.button("Lit (Blinn-Phong").clicked() {
            *material = Box::new(LitMaterialBuilder::new().build(device, queue));
            changed = true;
        }
        if ui.button("Pbr").clicked() {
            *material = Box::new(PbrMaterialBuilder::new().build(device, queue));
            changed = true;
        }
    });
    changed
}
pub fn lights_gui(ui: &mut Ui, lights: &mut Vec<Light>) -> bool {
    let mut changed = false;
    let mut indices = vec![];
    ui.collapsing("Lights", |ui| {
        for (i, gpu_light) in lights.iter_mut().enumerate() {
            changed |= gpu_light.gui(ui);
            if ui.button("Remove Light").clicked() {
                changed = true;
                indices.push(i);
            }
        }
        for i in indices.iter().rev() {
            // Remove in reverse order
            lights.remove(*i);
        }
        ui.menu_button("Add Light", |ui| {
            if ui.button("Directional Light").clicked() {
                lights.push(DirectionalLight::default().into());
                changed = true;
            }
            if ui.button("Point Light").clicked() {
                lights.push(PointLight::default().into());
                changed = true;
            }
            if ui.button("Spot Light").clicked() {
                lights.push(SpotLight::default().into());
                changed = true;
            }
        });
    });
    changed || !indices.is_empty()
}

#[allow(clippy::float_cmp)]
pub fn drag_angle_clamp(ui: &mut Ui, radians: &mut f32, range: RangeInclusive<f32>) -> Response {
    let mut degrees = radians.to_degrees();
    let mut response = ui.add(
        DragValue::new(&mut degrees)
            .speed(1.0)
            .suffix("°")
            .clamp_range(range),
    );

    // only touch `*radians` if we actually changed the degree value
    if degrees != radians.to_degrees() {
        *radians = degrees.to_radians();
        response.changed = true;
    }

    response
}

pub fn color_edit<I: Into<Color> + From<Color> + Clone + Copy>(
    ui: &mut Ui,
    into_color: &mut I,
    label: &str,
) -> bool {
    let mut color: Color = (*into_color).into();
    let mut rgb: [f32; 3] = color.into();
    ui.horizontal(|ui| {
        ui.label(label);
        let response = ui.color_edit_button_rgb(&mut rgb);
        if response.changed() {
            color = rgb.into();
            *into_color = color.into();
        }
        response
    })
    .inner
    .changed()
}

pub fn quat_edit(ui: &mut Ui, rotation: &mut Quat, euler_rot: EulerRot) {
    ui.label("Rotation (Euler)");
    let mut eulers: Vec3 = rotation.to_euler(euler_rot).into();
    let mut rotation_changed = false;
    ui.horizontal(|ui| {
        ui.label("X");
        rotation_changed |= ui.drag_angle(&mut eulers.x).changed();
    });
    ui.horizontal(|ui| {
        ui.label("Y");
        rotation_changed |= ui.drag_angle(&mut eulers.y).changed();
    });
    ui.horizontal(|ui| {
        ui.label("Z");
        rotation_changed |= ui.drag_angle(&mut eulers.z).changed();
    });
    if rotation_changed {
        *rotation = Quat::from_euler(euler_rot, eulers.x, eulers.y, eulers.z);
    }
}

pub fn vec3_edit(ui: &mut Ui, v: &mut Vec3, label: &str) -> bool {
    ui.label(label);
    let mut changed = ui
        .horizontal(|ui| {
            ui.label("X");
            ui.add(DragValue::new(&mut v.x)).changed()
        })
        .inner;
    changed |= ui
        .horizontal(|ui| {
            ui.label("Y");
            ui.add(DragValue::new(&mut v.y)).changed()
        })
        .inner;
    changed |= ui
        .horizontal(|ui| {
            ui.label("Z");
            ui.add(DragValue::new(&mut v.z)).changed()
        })
        .inner;
    changed
}

pub fn array3_edit(ui: &mut Ui, v: &mut [f32; 3], label: &str) -> bool {
    ui.label(label);
    let mut changed = ui
        .horizontal(|ui| {
            ui.label("X");
            ui.add(DragValue::new(&mut v[0])).changed()
        })
        .inner;
    changed |= ui
        .horizontal(|ui| {
            ui.label("Y");
            ui.add(DragValue::new(&mut v[1])).changed()
        })
        .inner;
    changed |= ui
        .horizontal(|ui| {
            ui.label("Z");
            ui.add(DragValue::new(&mut v[2])).changed()
        })
        .inner;
    changed
}

pub fn float_edit(ui: &mut Ui, f: &mut f32, label: &str, range: RangeInclusive<f32>) -> bool {
    ui.horizontal(|ui| ui.add(Slider::new(f, range).text(label)).changed())
        .inner
}

pub fn direction_edit(ui: &mut Ui, v: &mut Vec3, label: &str) -> bool {
    let mut polar = cartesian_to_polar(*v);
    ui.horizontal(|ui| {
        ui.label(label);
        let mut changed = ui.drag_angle(&mut polar.x).changed();
        changed |= ui.drag_angle(&mut polar.y).changed();
        if changed {
            polar.x = polar.x.clamp(-PI * 0.5, PI * 0.5);
            polar.y = polar.y.clamp(-PI * 0.5, PI * 0.5);
            *v = polar_to_cartesian(polar).normalize();
        }
        changed
    })
    .inner
}

fn cartesian_to_polar(cartesian: Vec3) -> Vec2 {
    let length = cartesian.length();
    let normalized = cartesian / length;
    Vec2 {
        x: normalized.y.asin(),                  // latitude
        y: (normalized.x / normalized.z).atan(), // longitude
    }
}

fn polar_to_cartesian(polar: Vec2) -> Vec3 {
    let latitude = polar.x;
    let longitude = polar.y;
    Vec3 {
        x: latitude.cos() * longitude.sin(),
        y: latitude.sin(),
        z: latitude.cos() * longitude.cos(),
    }
}
