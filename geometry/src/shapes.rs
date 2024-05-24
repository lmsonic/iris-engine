use approx::abs_diff_eq;
use glam::{Mat2, Vec2, Vec3};

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
    #[must_use]
    pub fn is_inside_triangle(&self, point: Vec3) -> bool {
        // Calculate baricentric coordinates to check if it is inside the triangle
        let r = point - self.v1;
        let q1 = self.v2 - self.v1;
        let q2 = self.v3 - self.v1;
        let dot = q1.dot(q2);
        let coefficients = Mat2::from_cols(
            [q1.length_squared(), dot].into(),
            [dot, q2.length_squared()].into(),
        );
        let constants = Vec2::new(r.dot(q1), r.dot(q2));
        let weights = coefficients.inverse() * constants;
        weights.x >= 0.0 && weights.y >= 0.0 && weights.x + weights.y <= 1.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cuboid {
    pub size: Vec3,
}

impl Cuboid {
    #[must_use]
    pub const fn new(size: Vec3) -> Self {
        Self { size }
    }
    #[must_use]
    pub fn is_point_on_surface(&self, point: Vec3) -> bool {
        abs_diff_eq!(point.x, 0.0, epsilon = 1e-2)
            || abs_diff_eq!(point.x, self.size.x, epsilon = 1e-2)
            || abs_diff_eq!(point.y, 0.0, epsilon = 1e-2)
            || abs_diff_eq!(point.y, self.size.y, epsilon = 1e-2)
            || abs_diff_eq!(point.z, 0.0, epsilon = 1e-2)
            || abs_diff_eq!(point.z, self.size.z, epsilon = 1e-2)
    }
    #[must_use]
    pub fn is_point_inside(&self, point: Vec3) -> bool {
        point.x >= 0.0 && point.x <= self.size.x
            || point.y >= 0.0 && point.y <= self.size.y
            || point.z >= 0.0 && point.z <= self.size.z
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
    pub radius: Vec3,
}

impl Ellipsoid {
    #[must_use]
    pub const fn new(radius: Vec3) -> Self {
        Self { radius }
    }
    #[must_use]
    // Assuming point is on surface
    pub fn normal(&self, point: Vec3) -> Vec3 {
        Vec3::new(
            2.0 * point.x * (self.radius.x * self.radius.x),
            2.0 * point.y * (self.radius.y * self.radius.y),
            2.0 * point.z * (self.radius.z * self.radius.z),
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
