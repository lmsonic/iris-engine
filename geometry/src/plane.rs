use approx::{AbsDiffEq, RelativeEq};
use glam::{Mat3, Vec3, Vec4};

use crate::line::Line;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}
#[cfg(test)]
impl AbsDiffEq for Plane {
    type Epsilon = <Vec3 as approx::AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        Vec3::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.normal.abs_diff_eq(other.normal, epsilon)
            && self.distance.abs_diff_eq(&other.distance, 0.5)
    }
}

#[cfg(test)]
impl RelativeEq for Plane {
    fn default_max_relative() -> Self::Epsilon {
        Vec3::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.normal
            .relative_eq(&other.normal, epsilon, max_relative)
            && self
                .distance
                .relative_eq(&other.distance, epsilon, max_relative)
    }
}

impl Plane {
    #[must_use]
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        let normal = normal.normalize();
        Self {
            distance: -normal.dot(point),
            normal,
        }
    }
    #[must_use]
    pub fn from_vec4(value: Vec4) -> Self {
        let normal = value.truncate();
        let length = normal.length();
        let normal = normal / length;
        Self {
            normal,
            distance: value.w / length,
        }
    }

    #[must_use]
    pub fn homogeneous(&self) -> Vec4 {
        self.normal.extend(self.distance)
    }
    #[must_use]
    pub fn signed_distance_to_point(&self, point: Vec3) -> f32 {
        let homogeneous = self.homogeneous();
        let point = point.extend(1.0);
        homogeneous.dot(point)
    }

    #[must_use]
    pub fn intersection_with_line(&self, line: Line) -> Option<Vec3> {
        let v = line.direction.extend(0.0);
        let homogeneous = self.homogeneous();
        let den = homogeneous.dot(v);
        if den == 0.0 {
            // Line parallel to plane
            None
        } else {
            let s = line.start.extend(1.0);
            let num = homogeneous.dot(s);
            let t = -(num / den);
            let point = line.point(t);
            Some(point)
        }
    }
    #[must_use]
    pub fn intersection_with_planes(&self, p1: Self, p2: Self) -> Option<Vec3> {
        // Rows are the normals
        let matrix = Mat3::from_cols(self.normal, p1.normal, p2.normal).transpose();
        let constants = -Vec3::new(self.distance, p1.distance, p2.distance);
        let determinant = matrix.determinant();
        if determinant == 0.0 {
            None
        } else {
            let result = matrix.inverse() * constants;
            Some(result)
        }
    }
    #[must_use]
    pub fn intersection_with_plane(&self, other: Self) -> Option<Line> {
        let direction = self.normal.cross(other.normal);
        let line_plane = Self {
            normal: direction.normalize(),
            distance: 0.0,
        };
        self.intersection_with_planes(other, line_plane)
            .map(|point| Line::new(point, direction))
    }
}
#[cfg(test)]
mod tests {}
