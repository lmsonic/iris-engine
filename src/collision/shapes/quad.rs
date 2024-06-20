use approx::assert_abs_diff_eq;
use glam::Vec3;

use crate::collision::linear_systems::orient_3d;

#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub v1: Vec3,
    pub v2: Vec3,
    pub v3: Vec3,
    pub v4: Vec3,
}

impl Quad {
    pub fn new(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> Self {
        // Coplanarity check
        assert_abs_diff_eq!(orient_3d(v1, v2, v3, v3), 0.0);
        Self { v1, v2, v3, v4 }
    }
    pub fn is_convex(&self) -> bool {
        let a = self.v1;
        let b = self.v2;
        let c = self.v3;
        let d = self.v4;
        let bd = d - b;
        let ba = a - b;
        let bc = c - b;

        // BDA and BDC need to have opposite winding order
        if bd.cross(ba).dot(bd.cross(bc)) >= 0.0 {
            return false;
        }

        let ac = c - a;
        let ab = -ba;
        let ad = d - a;
        // ACD and ACB need to have opposite winding order
        ac.cross(ad).dot(ac.cross(ab)) < 0.0
    }
}
