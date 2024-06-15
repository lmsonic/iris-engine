use egui::load::SizedTexture;
use egui::{Button, DragValue, Response, Slider, TextureId, Ui, Vec2};

use glam::Vec3;
use std::f32::consts::PI;
use std::ops::RangeInclusive;

use super::color::Color;
use super::light::{DirectionalLight, Light, PointLight, SpotLight};

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

pub fn lights_gui(ui: &mut Ui, lights: &mut Vec<Light>) -> bool {
    let mut changed = false;
    let mut indices = vec![];
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

pub fn vec3_edit(ui: &mut Ui, v: &mut Vec3, label: &str, range: RangeInclusive<f32>) -> bool {
    ui.label(label);
    let mut changed = ui
        .add(Slider::new(&mut v.x, range.clone()).text("X"))
        .changed();
    changed |= ui
        .add(Slider::new(&mut v.y, range.clone()).text("Y"))
        .changed();
    changed |= ui.add(Slider::new(&mut v.z, range).text("Z")).changed();
    changed
}
pub fn array3_edit(ui: &mut Ui, v: &mut [f32; 3], label: &str, range: RangeInclusive<f32>) -> bool {
    ui.label(label);

    let mut changed = ui
        .add(Slider::new(&mut v[0], range.clone()).text("X"))
        .changed();
    changed |= ui
        .add(Slider::new(&mut v[1], range.clone()).text("Y"))
        .changed();
    changed |= ui.add(Slider::new(&mut v[2], range).text("Z")).changed();
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
