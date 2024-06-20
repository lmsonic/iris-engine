use std::f32::consts::FRAC_PI_2;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3, Vec4};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
};

use crate::{collision::shapes::Plane, visibility::frustum::Frustum, GpuSendable};

#[derive(Debug, Clone, Copy, Default)]
pub struct OrbitCamera {
    camera: PerspectiveCamera,
    orbit_radius: f32,
    yaw: f32,
    pitch: f32,
    drag: bool,
    last_mouse_pos: PhysicalPosition<f64>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuCamera {
    projection: Mat4,
    view: Mat4,
    inv_view: Mat4,
    position: Vec3,
    _pad: f32,
}

impl GpuSendable<GpuCamera> for OrbitCamera {
    fn to_gpu(&self) -> GpuCamera {
        let view = self.view();
        GpuCamera {
            projection: self.camera.matrix_rh(),
            view,
            inv_view: view.inverse().transpose(),
            position: self.position(),
            _pad: 0.0,
        }
    }
}

impl OrbitCamera {
    pub fn new(orbit_radius: f32, aspect_ratio: f32) -> Self {
        let mut camera = Self {
            orbit_radius,
            ..Default::default()
        };
        camera.set_aspect_ratio(aspect_ratio);
        camera
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
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.camera.aspect_ratio = aspect_ratio;
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match *event {
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

    pub fn frustum(&self) -> Frustum {
        // TODO: align frustum with view
        self.camera.frustum(false, false)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PerspectiveCamera {
    pub fov_y: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            fov_y: f32::to_radians(45.0),
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

impl PerspectiveCamera {
    pub const fn new(fov_y: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            fov_y,
            aspect_ratio,
            near,
            far,
        }
    }

    pub fn matrix_lh(&self) -> Mat4 {
        Mat4::perspective_lh(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    pub fn matrix_rh(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    pub fn matrix_gl(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    pub fn matrix_infinite_lh(&self) -> Mat4 {
        Mat4::perspective_infinite_lh(self.fov_y, self.aspect_ratio, self.near)
    }

    pub fn matrix_infinite_rh(&self) -> Mat4 {
        Mat4::perspective_infinite_rh(self.fov_y, self.aspect_ratio, self.near)
    }

    pub fn matrix_infinite_gl(&self) -> Mat4 {
        let f = 1.0 / f32::tan(0.5 * self.fov_y);
        Mat4::from_cols(
            Vec4::new(f / self.aspect_ratio, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, -1.0, -1.0),
            Vec4::new(0.0, 0.0, -2.0 * self.near, 0.0),
        )
    }

    pub fn matrix_infinite_reverse_lh(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_lh(self.fov_y, self.aspect_ratio, self.near)
    }

    pub fn matrix_infinite_reverse_rh(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_rh(self.fov_y, self.aspect_ratio, self.near)
    }

    pub fn matrix_infinite_reverse_gl(&self) -> Mat4 {
        let f = 1.0 / f32::tan(0.5 * self.fov_y);
        Mat4::from_cols(
            Vec4::new(f / self.aspect_ratio, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, -1.0),
            Vec4::new(0.0, 0.0, 2.0 * self.near, 0.0),
        )
    }

    pub fn focal_distance(&self) -> f32 {
        ((self.fov_y * 0.5).tan() * self.aspect_ratio).recip()
    }

    pub fn frustum(&self, is_left_handed: bool, infinite: bool) -> Frustum {
        let focal_distance = self.focal_distance();
        let aspect_ratio = self.aspect_ratio;
        let handedness = if is_left_handed { 1.0 } else { -1.0 };
        let top = Plane::new(
            Vec3::ZERO,
            Vec3::new(0.0, -focal_distance, handedness * aspect_ratio.recip()),
        );
        let bottom = Plane::new(
            Vec3::ZERO,
            Vec3::new(0.0, focal_distance, handedness * aspect_ratio.recip()),
        );
        let right = Plane::new(Vec3::ZERO, Vec3::new(-focal_distance, 0.0, handedness));
        let left = Plane::new(Vec3::ZERO, Vec3::new(focal_distance, 0.0, handedness));

        let near = Plane::new(
            Vec3::new(0.0, 0.0, handedness * self.near),
            handedness * Vec3::Z,
        );
        let far = if infinite {
            None
        } else {
            Some(Plane::new(
                Vec3::new(0.0, 0.0, handedness * self.far),
                handedness * Vec3::NEG_Z,
            ))
        };
        Frustum {
            near,
            far,
            left,
            right,
            bottom,
            top,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OrthographicCamera {
    pub size: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl OrthographicCamera {
    pub const fn new(size: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            size,
            aspect_ratio,
            near,
            far,
        }
    }

    pub fn matrix_lh(&self) -> Mat4 {
        let half_width = self.size * 0.5;
        let half_height = self.size * 0.5 * self.aspect_ratio;

        Mat4::orthographic_lh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            self.near,
            self.far,
        )
    }

    pub fn matrix_rh(&self) -> Mat4 {
        let half_width = self.size * 0.5;
        let half_height = self.size * 0.5 * self.aspect_ratio;

        Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            self.near,
            self.far,
        )
    }

    pub fn matrix_gl(&self) -> Mat4 {
        let half_width = self.size * 0.5;
        let half_height = self.size * 0.5 * self.aspect_ratio;

        Mat4::orthographic_rh_gl(
            -half_width,
            half_width,
            -half_height,
            half_height,
            self.near,
            self.far,
        )
    }

    pub fn frustum(&self, is_left_handed: bool) -> Frustum {
        let half_size = self.size * 0.5;

        let top = Plane::new(
            Vec3::new(0.0, self.aspect_ratio * half_size, 0.0),
            Vec3::NEG_Y,
        );
        let bottom = Plane::new(Vec3::new(0.0, -self.aspect_ratio * half_size, 0.0), Vec3::Y);
        let right = Plane::new(Vec3::new(half_size, 0.0, 0.0), Vec3::NEG_X);
        let left = Plane::new(Vec3::new(-half_size, 0.0, 0.0), Vec3::X);
        let handedness = if is_left_handed { 1.0 } else { -1.0 };

        let near = Plane::new(
            Vec3::new(0.0, 0.0, handedness * self.near),
            handedness * Vec3::Z,
        );
        let far = Plane::new(
            Vec3::new(0.0, 0.0, handedness * self.far),
            handedness * Vec3::NEG_Z,
        );
        Frustum::new(near, far, left, right, bottom, top)
    }
}
