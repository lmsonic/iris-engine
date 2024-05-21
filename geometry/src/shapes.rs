use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub v1: Vec3,
    pub v2: Vec3,
    pub v3: Vec3,
}

impl Triangle {
    #[must_use]
    pub const fn new(v1: Vec3, v2: Vec3, v3: Vec3) -> Self {
        Self { v1, v2, v3 }
    }
    #[must_use]
    pub fn normal(&self) -> Vec3 {
        (self.v2 - self.v1).cross(self.v3 - self.v1)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cuboid {
    pub min: Vec3,
    pub max: Vec3,
}

impl Cuboid {
    #[must_use]
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub radius: f32,
}

impl Sphere {
    #[must_use]
    pub const fn new(radius: f32) -> Self {
        Self { radius }
    }
    #[must_use]
    // Assuming point is on surface
    pub fn normal(&self, point: Vec3) -> Vec3 {
        Vec3::new(2.0 * point.x, 2.0 * point.y, 2.0 * point.z)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ellipsoid {
    pub semiaxis_xy: f32,
    pub semiaxis_xz: f32,
}

impl Ellipsoid {
    #[must_use]
    pub const fn new(semiaxis_xy: f32, semiaxis_xz: f32) -> Self {
        Self {
            semiaxis_xy,
            semiaxis_xz,
        }
    }
    #[must_use]
    // Assuming point is on surface
    pub fn normal(&self, point: Vec3) -> Vec3 {
        Vec3::new(
            2.0 * point.x,
            2.0 * self.semiaxis_xy * self.semiaxis_xy * point.y,
            2.0 * self.semiaxis_xz * self.semiaxis_xz * point.z,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cylinder {
    pub radius_x: f32,
    pub radius_y: f32,
    pub height: f32,
}

impl Cylinder {
    #[must_use]
    pub const fn new(radius_x: f32, radius_y: f32, height: f32) -> Self {
        Self {
            radius_x,
            radius_y,
            height,
        }
    }
    // Assuming point is on surface
    #[must_use]
    pub fn normal(&self, point: Vec3) -> Vec3 {
        Vec3::new(
            2.0 * self.radius_y * point.x,
            2.0 * self.radius_x * point.y,
            if point.z <= 0.0 {
                -1.0
            } else if point.z > self.height {
                1.0
            } else {
                0.0
            },
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Torus {
    pub inner_radius: f32,
    pub outer_radius: f32,
}

impl Torus {
    #[must_use]
    pub const fn new(inner_radius: f32, outer_radius: f32) -> Self {
        Self {
            inner_radius,
            outer_radius,
        }
    }
    #[must_use]
    // Assuming point is on surface
    pub fn normal(&self, point: Vec3) -> Vec3 {
        let recip_sqrt_term = 2.0 * self.inner_radius * point.x.hypot(point.y).recip();
        Vec3::new(
            2.0 * point.x - point.x * recip_sqrt_term,
            2.0 * point.y - point.y * recip_sqrt_term,
            2.0 * point.z,
        )
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    #[test]
    fn exercise6() {
        let point = Vec3::new(-1.0, 2.0, 14.0);
        // Equation 2x^2 + 3y^2 -z = 0
        let gradient = |p: Vec3| Vec3::new(4.0 * p.x, 6.0 * p.y, -1.0);
        assert!(
            gradient(point)
                .normalize()
                .distance(Vec3::new(-0.315, 0.946, -0.0788))
                < 0.001,
        );
    }
}
