use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Quat, Vec3};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct OrbitCamera {
    orbit_radius: f32,
    yaw: f32,
    pitch: f32,
    drag: bool,
    last_mouse_pos: PhysicalPosition<f64>,
}

impl OrbitCamera {
    pub fn new(orbit_radius: f32) -> Self {
        Self {
            orbit_radius,
            ..Default::default()
        }
    }

    pub fn position(&self) -> Vec3 {
        Quat::from_rotation_y(self.yaw)
            * Quat::from_rotation_x(self.pitch)
            * Vec3::Z
            * self.orbit_radius
    }
    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), Vec3::ZERO, Vec3::Y)
    }

    pub fn input(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                const SENSITIVITY: f32 = 0.0005;
                if self.drag {
                    let delta_y = (position.y - self.last_mouse_pos.y) as f32 * SENSITIVITY;
                    let delta_x = (position.x - self.last_mouse_pos.x) as f32 * SENSITIVITY;
                    self.yaw -= delta_x;
                    self.pitch -= delta_y;
                    self.pitch = self.pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
                    return true;
                }
                self.last_mouse_pos = position;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Middle {
                    match state {
                        ElementState::Pressed => self.drag = true,
                        ElementState::Released => self.drag = false,
                    }
                    // if self.drag != last_drag {
                    return true;
                    // }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                const SENSITIVITY: f32 = 0.1;

                match delta {
                    MouseScrollDelta::LineDelta(_, y) => self.orbit_radius -= y * SENSITIVITY,
                    MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => {
                        self.orbit_radius -= y as f32 * SENSITIVITY;
                    }
                }
                self.orbit_radius = self.orbit_radius.max(0.1);
                return true;
            }
            _ => {}
        }
        false
    }
}
