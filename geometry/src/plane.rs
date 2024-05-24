use approx::{abs_diff_eq, AbsDiffEq, RelativeEq};
use glam::{Mat3, Vec3, Vec4};

use crate::line::Line;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl AbsDiffEq for Plane {
    type Epsilon = <Vec3 as approx::AbsDiffEq>::Epsilon;

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
    pub fn is_on_plane(&self, point: Vec3) -> bool {
        abs_diff_eq!(self.signed_distance_to_point(point), 0.0, epsilon = 0.2)
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
            .map(|point| Line::line(point, direction))
    }
}
#[cfg(test)]
mod tests {

    use std::ops::RangeInclusive;

    use glam::Vec3;
    use proptest::prop_assume;
    use proptest::prop_compose;
    use proptest::proptest;
    use proptest::strategy::Strategy;

    use super::Plane;
    prop_compose! {
        fn any_vec3(range:RangeInclusive<f32>)
                    (x in range.clone(),y in range.clone(),z in range)
                    -> Vec3 {
            Vec3::new(x, y, z)
        }
    }
    prop_compose! {
        fn any_plane(range:RangeInclusive<f32>)
                    (point in any_vec3(range.clone()),
                        normal in any_vec3(range).prop_filter("normal needs to be able to be normalized",
                        |n|n.is_finite()))
                    -> Plane {

            Plane::new(point,normal)
        }
    }

    proptest! {
        #[test]
        fn test_distance_to_point(point in any_vec3(-1000.0..=1000.0), normal in any_vec3(-1000.0..=1000.0)){
            prop_assume!(normal.try_normalize().is_some());
            _test_distance_to_point(point, normal);
        }

        #[test]
        fn test_intersect_three_planes(p1 in any_plane(-1000.0..=1000.0), p2 in any_plane(-1000.0..=1000.0), p3 in any_plane(-1000.0..=1000.0)){
            _test_intersect_three_planes(p1,p2,p3);
        }
        #[test]
        fn test_intersect_two_planes(p1 in any_plane(-1000.0..=1000.0), p2 in any_plane(-1000.0..=1000.0)){
            _test_intersect_two_planes(p1,p2);
        }
    }

    fn _test_distance_to_point(point: Vec3, normal: Vec3) {
        let plane = Plane::new(point, normal);

        assert!(plane.is_on_plane(point));
    }

    fn _test_intersect_three_planes(p1: Plane, p2: Plane, p3: Plane) {
        if let Some(point) = p1.intersection_with_planes(p2, p3) {
            println!("{point}");
            assert!(p1.is_on_plane(point));
            assert!(p2.is_on_plane(point));
            assert!(p3.is_on_plane(point));
        }
    }
    fn _test_intersect_two_planes(p1: Plane, p2: Plane) {
        if let Some(line) = p1.intersection_with_plane(p2) {
            let end = line.start + line.direction;
            assert!(p1.is_on_plane(line.start));
            assert!(p1.is_on_plane(end));
            assert!(p2.is_on_plane(line.start));
            assert!(p2.is_on_plane(end));
        }
    }
}
