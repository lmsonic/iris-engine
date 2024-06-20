use approx::{abs_diff_eq, AbsDiffEq, RelativeEq};
use glam::{Mat3, Mat4, Vec3, Vec4};

use super::ray::Ray;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        let normal = normal.normalize();
        Self {
            distance: -normal.dot(point),
            normal,
        }
    }
    pub fn from_points(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        let normal = (p2 - p1).cross(p3 - p1).normalize();
        Self {
            distance: normal.dot(p1),
            normal,
        }
    }

    pub fn homogeneous(&self) -> Vec4 {
        Vec4::from(*self)
    }

    pub fn signed_distance_to(&self, point: Vec3) -> f32 {
        let homogeneous = self.homogeneous();
        let point = point.extend(1.0);
        homogeneous.dot(point)
    }
    pub fn contains(&self, point: Vec3) -> bool {
        abs_diff_eq!(self.signed_distance_to(point), 0.0)
    }

    pub fn closest_on_plane(&self, point: Vec3) -> Vec3 {
        let distance = self.signed_distance_to(point);
        point - distance * self.normal
    }

    pub fn intersection_with_planes(&self, p1: Self, p2: Self) -> Option<Vec3> {
        // Rows are the normals
        let matrix = Mat3::from_cols(self.normal, p1.normal, p2.normal).transpose();
        let constants = -Vec3::new(self.distance, p1.distance, p2.distance);
        let determinant = matrix.determinant();
        if abs_diff_eq!(determinant, 0.0, epsilon = 1e-3) {
            None
        } else {
            let result = matrix.inverse() * constants;
            Some(result)
        }
    }

    pub fn intersection_with_plane(&self, other: Self) -> Option<Ray> {
        let direction = self.normal.cross(other.normal);
        let line_plane = Self {
            normal: direction.normalize(),
            distance: 0.0,
        };
        self.intersection_with_planes(other, line_plane)
            .map(|point| Ray::new(point, direction))
    }

    pub fn clip_projection_matrix(&self, mut matrix: Mat4) -> Mat4 {
        let inverse = matrix.inverse();
        let clip_plane = self.homogeneous();
        // transform into clip space
        let proj_clip_plane = inverse.transpose() * clip_plane;
        // Corner opposite to clip plane
        let proj_corner = Vec4::new(
            proj_clip_plane.x.signum(),
            proj_clip_plane.y.signum(),
            1.0,
            1.0,
        );
        // transform into camera space
        let corner = inverse * proj_corner;
        let r4 = matrix.row(3);
        // find the scale factor u for the plane to force far_plane.dot(corner) == 0 : far plane to contain the corner
        let u = 2.0 * r4.dot(corner) / clip_plane.dot(corner);
        let new_r3 = u * clip_plane - r4;

        matrix.x_axis[2] = new_r3.x;
        matrix.y_axis[2] = new_r3.y;
        matrix.z_axis[2] = new_r3.z;
        matrix.w_axis[2] = new_r3.w;
        matrix
    }
}

impl From<Vec4> for Plane {
    fn from(value: Vec4) -> Self {
        let normal = value.truncate();
        let length = normal.length();
        let normal = normal / length;
        Self {
            normal,
            distance: value.w / length,
        }
    }
}

impl From<Plane> for Vec4 {
    fn from(value: Plane) -> Self {
        value.normal.extend(value.distance)
    }
}

impl AbsDiffEq for Plane {
    type Epsilon = <Vec3 as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        Vec3::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.normal.abs_diff_eq(other.normal, epsilon)
            && self.distance.abs_diff_eq(&other.distance, epsilon)
    }
}

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
#[cfg(test)]
mod tests {

    use std::ops::RangeInclusive;

    use approx::assert_abs_diff_eq;
    use glam::Vec3;
    use proptest::prop_compose;
    use proptest::proptest;

    use crate::tests::any_normal;
    use crate::tests::any_vec3;

    use super::Plane;

    prop_compose! {
        fn any_plane(range:RangeInclusive<f32>)
                    (point in any_vec3(range),
                        normal in any_normal())
                    -> Plane {

            Plane::new(point,normal)
        }
    }
    const RANGE: RangeInclusive<f32> = -100.0..=100.0;
    proptest! {
        #[test]
        fn plane_center_is_on_plane(point in any_vec3(RANGE), normal in any_normal()){
            _plane_center_is_on_plane(point, normal);
        }
        #[test]
        fn closest_on_plane(plane in any_plane(RANGE), point in any_vec3(RANGE)){
            _closest_on_plane(plane,point);
        }

        #[test]
        fn intersect_three_planes(p1 in any_plane(RANGE), p2 in any_plane(RANGE), p3 in any_plane(RANGE)){
            _intersect_three_planes(p1,p2,p3);
        }
        #[test]
        fn intersect_two_planes(p1 in any_plane(RANGE), p2 in any_plane(RANGE)){
            _intersect_two_planes(p1,p2);
        }
    }

    fn _plane_center_is_on_plane(point: Vec3, normal: Vec3) {
        let plane = Plane::new(point, normal);
        assert_abs_diff_eq!(plane.signed_distance_to(point), 0.0, epsilon = 1e-3);
    }
    fn _closest_on_plane(plane: Plane, point: Vec3) {
        let closest_point = plane.closest_on_plane(point);
        assert_abs_diff_eq!(plane.signed_distance_to(closest_point), 0.0, epsilon = 1e-3);
    }

    fn _intersect_three_planes(p1: Plane, p2: Plane, p3: Plane) {
        if let Some(point) = p1.intersection_with_planes(p2, p3) {
            assert_abs_diff_eq!(p1.signed_distance_to(point), 0.0, epsilon = 1e-1);
            assert_abs_diff_eq!(p2.signed_distance_to(point), 0.0, epsilon = 1e-1);
            assert_abs_diff_eq!(p3.signed_distance_to(point), 0.0, epsilon = 1e-1);
        }
    }
    fn _intersect_two_planes(p1: Plane, p2: Plane) {
        if let Some(line) = p1.intersection_with_plane(p2) {
            let end = line.start + line.direction;
            assert_abs_diff_eq!(p1.signed_distance_to(line.start), 0.0, epsilon = 1e-1);
            assert_abs_diff_eq!(p1.signed_distance_to(end), 0.0, epsilon = 1e-1);
            assert_abs_diff_eq!(p2.signed_distance_to(line.start), 0.0, epsilon = 1e-1);
            assert_abs_diff_eq!(p2.signed_distance_to(end), 0.0, epsilon = 1e-1);
        }
    }
}
