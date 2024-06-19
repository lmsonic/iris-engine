use approx::assert_abs_diff_eq;
use glam::{Vec2, Vec3, Vec3Swizzles};

#[derive(Clone, Copy, Debug)]
pub struct Cylinder {
    pub radius_x: f32,
    pub radius_y: f32,
    pub height: f32,
}

impl Cylinder {
    pub const fn new(radius_x: f32, radius_y: f32, height: f32) -> Self {
        Self {
            radius_x,
            radius_y,
            height,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn equation(self, p: Vec3) -> f32 {
        let p_xy = p.xy();
        let radius_xy = Vec2::new(self.radius_x, self.radius_y);
        (p_xy * p_xy).dot((radius_xy * radius_xy).recip()) - 1.0
    }
    #[allow(dead_code)]
    pub(crate) fn gradient(&self, p: Vec3) -> Vec3 {
        let p_xy = p.xy();
        let radius_xy = Vec2::new(self.radius_x, self.radius_y);
        let v_xy = 2.0 * (p_xy * radius_xy.recip());
        v_xy.extend(if p.z <= 0.0 {
            -1.0
        } else if p.z >= self.height {
            1.0
        } else {
            0.0
        })
    }

    // Assuming point is on surface
    #[allow(dead_code)]
    pub(crate) fn normal(&self, point: Vec3) -> Vec3 {
        assert_abs_diff_eq!(self.equation(point), 0.0, epsilon = 1e-1);
        Self::gradient(self, point)
    }
}
