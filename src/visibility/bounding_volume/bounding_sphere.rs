use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

impl BoundingSphere {
    pub fn from_points(points: &[Vec3]) -> Self {
        let center = center_of_points(points);
        let mut radius_squared = 0.0;

        for point in points {
            // Get squared version to avoid unnecessary sqrt calls
            let distance_squared = point.distance_squared(center);
            if distance_squared > radius_squared {
                radius_squared = distance_squared;
            }
        }

        Self {
            center,
            radius: radius_squared.sqrt(),
        }
    }
    pub fn contains(&self, point: Vec3) -> bool {
        self.center.distance_squared(point) < self.radius * self.radius
    }
}

pub fn center_of_points(points: &[Vec3]) -> Vec3 {
    assert!(
        !points.is_empty(),
        "cannot compute the center of an empty mesh"
    );

    let denom = 1.0 / points.len() as f32;
    points.iter().fold(Vec3::ZERO, |acc, point| acc + *point) * denom
}

// Translation operations

impl AddAssign<Vec3> for BoundingSphere {
    fn add_assign(&mut self, rhs: Vec3) {
        *self = *self + rhs;
    }
}
impl SubAssign<Vec3> for BoundingSphere {
    fn sub_assign(&mut self, rhs: Vec3) {
        *self = *self - rhs;
    }
}

impl Add<Vec3> for BoundingSphere {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            center: self.center + rhs,
            radius: self.radius,
        }
    }
}
impl Add<BoundingSphere> for Vec3 {
    type Output = BoundingSphere;

    fn add(self, rhs: BoundingSphere) -> Self::Output {
        Self::Output {
            center: rhs.center + self,
            radius: rhs.radius,
        }
    }
}

impl Sub<Vec3> for BoundingSphere {
    type Output = Self;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            center: self.center - rhs,
            radius: self.radius,
        }
    }
}

// Scale operations

impl MulAssign<f32> for BoundingSphere {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Mul<f32> for BoundingSphere {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            center: self.center,
            radius: self.radius * rhs,
        }
    }
}
impl Mul<BoundingSphere> for f32 {
    type Output = BoundingSphere;

    fn mul(self, rhs: BoundingSphere) -> Self::Output {
        Self::Output {
            center: rhs.center,
            radius: rhs.radius * self,
        }
    }
}
