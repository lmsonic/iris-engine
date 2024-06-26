use approx::abs_diff_eq;
use glam::{Mat2, Vec2, Vec3};

use super::{
    intersections::{
        ray_intersect_cuboid, ray_intersect_cylinder, ray_intersect_ellipsoid, ray_intersect_plane,
        ray_intersect_sphere, ray_intersect_triangle,
    },
    plane::Plane,
    shapes::{Cuboid, Cylinder, Ellipsoid, Sphere, Triangle},
};

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub start: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(start: Vec3, direction: Vec3) -> Self {
        Self {
            start,
            direction: direction.normalize(),
        }
    }

    pub fn point(&self, t: f32) -> Vec3 {
        self.start + t * self.direction
    }

    pub fn distance_to(&self, point: Vec3) -> f32 {
        let delta = point - self.start;
        delta.reject_from(self.direction).length()
    }

    pub fn closest_point(&self, point: Vec3) -> Vec3 {
        let t = self.closest_t(point);
        self.point(t)
    }

    pub fn closest_t(&self, point: Vec3) -> f32 {
        (point - self.start).dot(self.direction) / self.direction.length_squared()
    }

    pub fn distance_to_line(&self, other: Self) -> f32 {
        let (t1, t2) = self.closest_ts(other);
        let point1 = self.point(t1);
        let point2 = other.point(t2);
        point1.distance(point2)
    }

    fn closest_ts(&self, other: Self) -> (f32, f32) {
        let v1_sqr = self.direction.length_squared();
        let v2_sqr = other.direction.length_squared();
        let dot = self.direction.dot(other.direction);
        let coefficient_matrix = Mat2::from_cols_array_2d(&[[v1_sqr, dot], [-dot, -v2_sqr]]);

        let delta = other.start - self.start;
        let constants = Vec2::new(delta.dot(self.direction), delta.dot(other.direction));

        let determinant = coefficient_matrix.determinant();
        if abs_diff_eq!(determinant, 0.0, epsilon = 1e-1) {
            // Parallel lines, any point will do
            (self.closest_t(self.start), 0.0)
        } else {
            // Skew lines
            let inverse = coefficient_matrix.inverse();
            let t = inverse * constants;
            (t.x, t.y)
        }
    }
    pub fn intersect_plane(&self, plane: Plane) -> Option<Vec3> {
        ray_intersect_plane(*self, plane)
    }
    pub fn intersect_triangle(&self, triangle: Triangle) -> Option<Vec3> {
        ray_intersect_triangle(*self, triangle)
    }
    pub fn intersect_cuboid(&self, cuboid: Cuboid) -> Option<Vec3> {
        ray_intersect_cuboid(*self, cuboid)
    }
    pub fn intersect_sphere(&self, sphere: Sphere) -> Option<Vec3> {
        ray_intersect_sphere(*self, sphere)
    }
    pub fn intersect_ellipsoid(&self, ellipsoid: Ellipsoid) -> Option<Vec3> {
        ray_intersect_ellipsoid(*self, ellipsoid)
    }
    pub fn intersect_cylinder(&self, cylinder: Cylinder) -> Option<Vec3> {
        ray_intersect_cylinder(*self, cylinder)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;

    use approx::assert_relative_eq;
    use glam::Vec3;
    use proptest::{prop_compose, proptest};

    use crate::tests::{any_normal, any_vec3};

    use super::Ray;

    prop_compose! {
        fn any_ray(range:RangeInclusive<f32>)
                    (start in any_vec3(range),
                        direction in any_normal())
                    -> Ray {

            Ray::new(start,direction)
        }
    }
    const RANGE: RangeInclusive<f32> = -1000.0..=1000.0;

    proptest! {
        #[test]
        fn closest_point(ray in any_ray(RANGE),point in any_vec3(RANGE)){
            _closest_point(ray, point);
        }
        #[test]
        fn distance_to_point(ray in any_ray(RANGE),point in any_vec3(RANGE)){
            _distance_to_point(ray, point);
        }
        #[test]
        fn closest_points(ray1 in any_ray(RANGE),ray2 in any_ray(RANGE)){
            _closest_points(ray1, ray2);
        }
    }

    fn _closest_point(ray: Ray, point: Vec3) {
        // Tests if it is a local minumum
        let closest_t = ray.closest_t(point);
        let closest_point = ray.point(closest_t);
        let distance_to_closest = ray.distance_to(point);

        assert_relative_eq!(
            distance_to_closest,
            closest_point.distance(point),
            max_relative = 0.99
        );
    }
    fn _distance_to_point(line: Ray, point: Vec3) {
        // Tests if it is a local minumum
        let closest_point = line.closest_point(point);

        assert_relative_eq!(
            closest_point.distance(point),
            line.distance_to(point),
            max_relative = 0.99
        );
    }
    fn _closest_points(ray1: Ray, ray2: Ray) {
        // Tests if it is a local minumum
        let distance = ray1.distance_to_line(ray2);
        let (t1, t2) = ray1.closest_ts(ray2);
        let point1 = ray1.point(t1);
        let point2 = ray2.point(t2);

        assert_relative_eq!(ray1.distance_to(point2), distance, max_relative = 0.99);
        assert_relative_eq!(ray2.distance_to(point1), distance, max_relative = 0.99);
    }
}
