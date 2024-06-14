use super::{
    plane::Plane,
    ray::Ray,
    root_finding::solve_quadratic,
    shapes::{Cuboid, Cylinder, Ellipsoid, Sphere, Triangle},
};
use approx::abs_diff_eq;
use glam::Vec3;

#[allow(clippy::multiple_inherent_impl)]
impl Ray {
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
            (t > 0.0).then(|| self.point(t))
        }
    }

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
        Some(self.point(t))
    }

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

        let m = ellipse.radius.x / ellipse.radius.y;
        let n = ellipse.radius.x / ellipse.radius.z;
        let m2 = m * m;
        let n2 = n * n;
        let r2 = ellipse.radius.x * ellipse.radius.x;

        let a = n2.mul_add(vz2, m2.mul_add(vy2, vx2));
        let b = 2.0 * (n2 * sz).mul_add(vz, sx.mul_add(vx, m2 * sy * vy));
        let c = n2.mul_add(sz2, m2.mul_add(sy2, sx2)) - r2;

        let solutions = solve_quadratic(a, b, c);
        let t = solutions
            .into_iter()
            .filter(|x| *x > 0.0)
            .min_by(f32::total_cmp)?;

        Some(self.point(t))
    }

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

        let a = m2.mul_add(vy2, vx2);
        let b = 2.0 * sx.mul_add(vx, m2 * sy * vy);
        let c = r.mul_add(-r, m2.mul_add(sy2, sx2));
        let solutions = solve_quadratic(a, b, c);
        let t = solutions
            .into_iter()
            .filter(|x| *x > 0.0)
            .min_by(f32::total_cmp)?;
        let point = self.point(t);
        if point.z < 0.0 || point.z > cylinder.height {
            return None;
        }
        Some(point)
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

    use crate::geometry::shapes::Cuboid;
    use crate::geometry::{
        plane::Plane,
        ray::Ray,
        shapes::{Cylinder, Ellipsoid, Sphere, Triangle},
    };
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
        fn test_intersect_cuboid(mut line in any_ray(RANGE), size in any_vec3(0.0..=100.0)){
            line.start = line.start.min(size * 1.1);
            _test_intersect_cuboid(line, Cuboid::new(size));

        }
        #[test]
        fn test_intersect_sphere(mut line in any_ray(RANGE), radius in 0.0..=100.0_f32){
            line.start = line.start.min(Vec3::ONE * radius * 1.1);
            _test_intersect_sphere(line, Sphere::new(radius));

        }
        #[test]
        fn test_intersect_ellipse(mut line in any_ray(RANGE), radius in any_vec3(0.0..=100.0_f32)){
            line.start = line.start.min(radius * 1.1);
            _test_intersect_ellipse(line, Ellipsoid::new(radius));

        }
        #[test]
        fn test_intersect_cylinder(mut line in any_ray(RANGE), size in any_vec3(0.0..=100.0_f32)){
            line.start = line.start.min(size * 1.1);
            _test_intersect_cylinder(line, Cylinder::new(size.x,size.y,size.z));

        }
        // #[test]
        // fn test_intersect_torus(line in any_ray(RANGE), inner_radius in 0.1..=100.0_f32,outer_radius in 0.1..=100.0_f32){
        //     prop_assume!(inner_radius>outer_radius);
        //     _test_intersect_torus(line, Torus::new(inner_radius,outer_radius));

        // }
    }

    fn _test_intersect_plane(ray: Ray, plane: Plane) {
        if let Some(point) = ray.intersect_plane(plane) {
            assert_abs_diff_eq!(plane.signed_distance_to_point(point), 0.0, epsilon = 0.1);
            assert_abs_diff_eq!(ray.distance_to_point(point), 0.0, epsilon = 0.1);
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

            assert_abs_diff_eq!(point.length(), sphere.radius, epsilon = 0.05);

            let opposite_ray = Ray::new(ray.start, -ray.direction);
            let intersect = opposite_ray.intersect_sphere(sphere);
            assert!(intersect.is_none());
        }
    }
    fn _test_intersect_ellipse(ray: Ray, ellipse: Ellipsoid) {
        if let Some(point) = ray.intersect_ellipsoid(ellipse) {
            assert_abs_diff_eq!(ray.distance_to_point(point), 0.0, epsilon = 1e-2);
        }
    }
    fn _test_intersect_cylinder(ray: Ray, cylinder: Cylinder) {
        if let Some(point) = ray.intersect_cylinder(cylinder) {
            assert_abs_diff_eq!(ray.distance_to_point(point), 0.0, epsilon = 1e-3);

            assert_abs_diff_eq!(cylinder.equation(point), 0.0, epsilon = 1e-3);
            assert!(point.z >= 0.0 && point.z <= cylinder.height);
        }
    }
}
