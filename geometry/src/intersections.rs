use approx::abs_diff_eq;
use glam::Vec3;

use crate::{
    plane::Plane,
    ray::Ray,
    root_finding::{solve_quadratic, solve_quartic},
    shapes::{Cuboid, Cylinder, Ellipsoid, Sphere, Torus, Triangle},
};

impl Ray {
    #[must_use]
    pub fn intersect_plane(&self, plane: Plane) -> Option<Vec3> {
        let v = self.direction.extend(0.0);
        let homogeneous = plane.homogeneous();
        let den = homogeneous.dot(v);
        if abs_diff_eq!(den, 0.0, epsilon = 1e-2) {
            // Line parallel to plane
            None
        } else {
            let s = self.start.extend(1.0);
            let num = homogeneous.dot(s);
            let t = -(num / den);
            if t > 0.0 {
                let point = self.point(t);
                Some(point)
            } else {
                None
            }
        }
    }
    #[must_use]
    pub fn intersect_triangle(&self, triangle: Triangle) -> Option<Vec3> {
        let normal = triangle.normal();
        let triangle_plane = Plane::new(triangle.v1, normal);

        if let Some(point) = self.intersect_plane(triangle_plane) {
            // Calculate baricentric coordinates to check if it is inside the triangle
            if triangle.is_inside_triangle(point) {
                return Some(point);
            }
        }
        None
    }
    #[must_use]
    #[allow(clippy::similar_names, clippy::useless_let_if_seq)]
    pub fn intersect_cuboid(&self, cuboid: Cuboid) -> Option<Vec3> {
        let plane_x0 = Plane::new(Vec3::ZERO, Vec3::X);
        let plane_xs = Plane::new(cuboid.size, Vec3::NEG_X);
        let plane_y0 = Plane::new(Vec3::ZERO, Vec3::Y);
        let plane_ys = Plane::new(cuboid.size, Vec3::NEG_Y);
        let plane_z0 = Plane::new(Vec3::ZERO, Vec3::Z);
        let plane_zs = Plane::new(cuboid.size, Vec3::NEG_Z);
        let mut planes: Vec<Plane> = Vec::with_capacity(3);
        if self.direction.x > 0.0 {
            planes.push(plane_x0);
        } else if self.direction.x < 0.0 {
            planes.push(plane_xs);
        }
        if self.direction.y > 0.0 {
            planes.push(plane_y0);
        } else if self.direction.y < 0.0 {
            planes.push(plane_ys);
        }
        if self.direction.z > 0.0 {
            planes.push(plane_z0);
        } else if self.direction.z < 0.0 {
            planes.push(plane_zs);
        }
        for plane in planes {
            if let Some(point) = self.intersect_plane(plane) {
                if cuboid.is_point_inside(point) {
                    return Some(point);
                }
            }
        }

        None
    }
    #[must_use]
    pub fn intersect_sphere(&self, sphere: Sphere) -> Option<Vec3> {
        let delta = self.start;
        let a = self.direction.length_squared(); // Should be 1.0
        let b = 2.0 * self.direction.dot(delta);
        let c = sphere
            .radius
            .mul_add(-sphere.radius, delta.length_squared());
        let solutions = solve_quadratic(a, b, c);
        let t = solutions.into_iter().min_by(f32::total_cmp)?;
        if t <= 0.0 {
            return None;
        }
        Some(self.point(t as f32))
    }
    #[must_use]
    #[allow(clippy::similar_names, clippy::many_single_char_names)]
    pub fn intersect_ellipsoid(&self, ellipse: Ellipsoid) -> Option<Vec3> {
        let vx = self.direction.x;
        let vy = self.direction.y;
        let vz = self.direction.z;
        let vx2 = vx * vx;
        let vy2 = vy * vy;
        let vz2 = vz * vz;

        let sx = self.start.x;
        let sy = self.start.y;
        let sz = self.start.z;
        let sx2 = sx * sx;
        let sy2 = sy * sy;
        let sz2 = sz * sz;

        let m = ellipse.horizontal;
        let n = ellipse.vertical;
        let m2 = m * m;
        let n2 = n * n;

        let a = vx2 + m2 * vy2 + n2 * vz2;
        let b = 2.0 * (sx * vx + m2 * sy * vy + n2 * sz * vz);
        let c = sx2 + m2 * sy2 + n2 * sz2;

        let solutions = solve_quadratic(a, b, c);
        let t = solutions
            .into_iter()
            .filter(|x| *x > 0.0)
            .min_by(f32::total_cmp)?;

        Some(self.point(t as f32))
    }

    #[must_use]
    #[allow(clippy::similar_names, clippy::many_single_char_names)]

    pub fn intersect_cylinder(&self, cylinder: Cylinder) -> Option<Vec3> {
        let m = cylinder.radius_x / cylinder.radius_y;
        let m2 = m * m;
        let r = cylinder.radius_x;
        let vx = self.direction.x;
        let vy = self.direction.y;
        let vx2 = vx * vx;
        let vy2 = vy * vy;

        let sx = self.start.x;
        let sy = self.start.y;
        let sx2 = sx * sx;
        let sy2 = sy * sy;

        let a = vx2 + m2 * vy2;
        let b = 2.0 * (sx * vx + m2 * sx * vy);
        let c = sx2 + m2 * sy2 - r * r;
        let solutions = solve_quadratic(a, b, c);
        let t = solutions
            .into_iter()
            .filter(|x| *x > 0.0)
            .min_by(f32::total_cmp)?;
        let point = self.point(t as f32);
        if point.z < 0.0 || point.z > cylinder.height {
            return None;
        }
        Some(point)
    }

    #[must_use]
    #[allow(clippy::similar_names, clippy::many_single_char_names)]

    pub fn intersect_torus(&self, torus: Torus) -> Option<Vec3> {
        let v2 = self.direction.length_squared();
        let v4 = v2 * v2;
        let vx2 = self.direction.x * self.direction.x;
        let vy2 = self.direction.y * self.direction.y;
        let vz = self.start.z;

        let r1_sqr = torus.inner_radius * torus.inner_radius;
        let r2_sqr = torus.outer_radius * torus.outer_radius;
        let r_sqr_diff = r1_sqr - r2_sqr;

        let sv = self.start.dot(self.direction);

        let s2 = self.start.length_squared();
        let sx2 = self.start.x * self.start.x;
        let sx4 = sx2 * sx2;
        let sy2 = self.start.y * self.start.y;
        let sy4 = sy2 * sy2;
        let sz = self.start.z;
        let sz2 = sz * sz;
        let sz4 = sz2 * sz2;

        let a = v4;
        let b = 4.0 * v2 * sv;
        let c = 2.0 * v2 * (s2 + r1_sqr - r2_sqr) - 4.0 * r1_sqr * (vx2 + vy2) + 4.0 * sv * sv;
        let d = 8.0 * r1_sqr * sz * vz - 4.0 * sv * (s2 - r1_sqr - r2_sqr);
        let e = sx4
            + sy4
            + sz4
            + r_sqr_diff * r_sqr_diff
            + 2.0 * (sx2 * sy2 + sz2 * r_sqr_diff + (sx2 + sy2) * (sz2 - r1_sqr - r2_sqr));
        let solutions = solve_quartic(a, b, c, d, e);
        let t = solutions
            .into_iter()
            .filter(|x| *x > 0.0)
            .min_by(f32::total_cmp)?;
        Some(self.point(t as f32))
    }
}

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;

    use approx::assert_abs_diff_eq;
    use glam::Vec3;
    use proptest::prop_compose;
    use proptest::proptest;
    use proptest::strategy::Strategy;

    use crate::shapes::Cuboid;
    use crate::shapes::Sphere;
    use crate::shapes::Triangle;
    use crate::{plane::Plane, ray::Ray};
    prop_compose! {
        fn any_vec3(range:RangeInclusive<f32>)
                    (x in range.clone(),y in range.clone(),z in range)
                    -> Vec3 {
            Vec3::new(x, y, z)
        }
    }
    prop_compose! {
        fn any_normal(range:RangeInclusive<f32>)
                    (n in any_vec3(range).prop_filter("normal needs to be able to be normalized",
                    |n|n.try_normalize().is_some()))
                    -> Vec3 {
            n
        }
    }

    prop_compose! {
        fn any_ray(range:RangeInclusive<f32>)
                    (start in any_vec3(range.clone()),
                    direction in any_normal(range))
                    -> Ray {

            Ray::new(start,direction)
        }
    }
    prop_compose! {
        fn any_plane(range:RangeInclusive<f32>)
                    (point in any_vec3(range.clone()),
                    normal in any_normal(range))
                    -> Plane {

            Plane::new(point,normal)
        }
    }
    prop_compose! {
        fn any_triangle(range:RangeInclusive<f32>)
                    (v1 in any_vec3(range.clone()),
                    v2 in any_vec3(range.clone()),
                    v3 in any_vec3(range))
                    -> Triangle {

            Triangle::new(v1, v2, v3)
        }
    }
    prop_compose! {
        fn any_cuboid(range:RangeInclusive<f32>)
                    (size in any_vec3(range))
                    -> Cuboid {

            Cuboid::new(size)
        }
    }
    const RANGE: RangeInclusive<f32> = -1000.0..=1000.0;
    proptest! {

        #[test]
        fn test_intersect_plane(line in any_ray(RANGE), plane in any_plane(RANGE)){
            _test_intersect_plane(line, plane);
        }
        #[test]
        fn test_intersect_triangle(line in any_ray(RANGE), triangle in any_triangle(RANGE)){
            _test_intersect_triangle(line, triangle);

        }
        #[test]
        fn test_intersect_cuboid(line in any_ray(RANGE), cuboid in any_cuboid(0.0..=100.0)){
            _test_intersect_cuboid(line, cuboid);

        }
        #[test]
        fn test_intersect_sphere(line in any_ray(RANGE), radius in 0.0..=100.0_f32){
            _test_intersect_sphere(line, Sphere::new(radius));

        }
    }

    fn _test_intersect_plane(ray: Ray, plane: Plane) {
        if let Some(point) = ray.intersect_plane(plane) {
            assert_abs_diff_eq!(plane.signed_distance_to_point(point), 0.0, epsilon = 0.02);
            assert_abs_diff_eq!(ray.distance_to_point(point), 0.0, epsilon = 0.02);
            let opposite_ray = Ray::new(ray.start, -ray.direction);
            let intersect = opposite_ray.intersect_plane(plane);
            assert!(intersect.is_none());
        }
    }
    fn _test_intersect_triangle(ray: Ray, triangle: Triangle) {
        if let Some(point) = ray.intersect_triangle(triangle) {
            let plane = Plane::new(triangle.v1, triangle.normal());
            assert_abs_diff_eq!(plane.signed_distance_to_point(point), 0.0, epsilon = 1e-1);
            assert_abs_diff_eq!(ray.distance_to_point(point), 0.0, epsilon = 1e-1);
            assert!(triangle.is_inside_triangle(point));
            let opposite_ray = Ray::new(ray.start, -ray.direction);
            let intersect = opposite_ray.intersect_triangle(triangle);
            assert!(intersect.is_none());
        }
    }
    fn _test_intersect_cuboid(ray: Ray, cuboid: Cuboid) {
        if let Some(point) = ray.intersect_cuboid(cuboid) {
            assert_abs_diff_eq!(ray.distance_to_point(point), 0.0, epsilon = 1e-1);
            assert!(cuboid.is_point_on_surface(point));
            assert!(cuboid.is_point_inside(point));
        }
    }
    fn _test_intersect_sphere(ray: Ray, sphere: Sphere) {
        if let Some(point) = ray.intersect_sphere(sphere) {
            assert_abs_diff_eq!(ray.distance_to_point(point), 0.0, epsilon = 1e-2);
            assert_abs_diff_eq!(sphere.radius, point.length(), epsilon = 1e-2);

            let opposite_ray = Ray::new(ray.start, -ray.direction);
            let intersect = opposite_ray.intersect_sphere(sphere);
            assert!(intersect.is_none());
        }
    }
}
