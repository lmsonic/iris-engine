use egui::{Context, DragValue, Response, Slider, Ui, Vec2};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use glam::Vec3;
use std::f32::consts::PI;
use std::ops::RangeInclusive;
use winit::event::WindowEvent;
use winit::window::Window;

use super::color::Color;
use super::light::{DirectionalLight, Light, PointLight, SpotLight};
use super::material::{LitMaterialBuilder, Material, PbrMaterialBuilder, UnlitMaterialBuilder};

pub trait GuiViewable {
    fn gui(&mut self, ui: &mut Ui, queue: &wgpu::Queue);
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

#[allow(missing_debug_implementations)]
pub struct EguiRenderer {
    state: State,
    renderer: Renderer,
}

impl EguiRenderer {
    #[allow(dead_code)]
    pub(crate) fn context(&self) -> &Context {
        self.state.egui_ctx()
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        output_depth_format: Option<wgpu::TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> Self {
        let egui_context = Context::default();

        let egui_state = State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );
        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
        );

        Self {
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub(crate) fn handle_input(
        &mut self,
        window: &Window,
        event: &WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }
    #[allow(dead_code)]
    pub(crate) fn ppp(&mut self, v: f32) {
        self.state.egui_ctx().set_pixels_per_point(v);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        window_surface_view: &wgpu::TextureView,
        screen_descriptor: &ScreenDescriptor,
        run_ui: impl FnOnce(&Context),
    ) {
        self.state
            .egui_ctx()
            .set_pixels_per_point(screen_descriptor.pixels_per_point);

        let raw_input = self.state.take_egui_input(window);
        let full_output = self.state.egui_ctx().run(raw_input, |_ui| {
            run_ui(self.state.egui_ctx());
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
        for (id, image_delta) in full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, id, &image_delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, screen_descriptor);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("egui main render pass"),
            occlusion_query_set: None,
        });
        self.renderer.render(&mut rpass, &tris, screen_descriptor);
        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x);
        }
    }
}
