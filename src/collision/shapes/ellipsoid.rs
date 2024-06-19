use approx::assert_abs_diff_eq;
use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Ellipsoid {
    pub radius: Vec3,
}

impl Ellipsoid {
    pub const fn new(radius: Vec3) -> Self {
        Self { radius }
    }
    #[allow(dead_code)]
    pub(crate) fn equation(self, p: Vec3) -> f32 {
        (p * p).dot((self.radius * self.radius).recip()) - 1.0
    }
    #[allow(dead_code)]
    pub(crate) fn gradient(&self, p: Vec3) -> Vec3 {
        2.0 * p * (self.radius * self.radius).recip()
    }

    // Assuming point is on surface
    #[allow(dead_code)]
    pub(crate) fn normal(&self, point: Vec3) -> Vec3 {
        assert_abs_diff_eq!(self.equation(point), 0.0, epsilon = 1e-1);
        Self::gradient(self, point)
    }
}
