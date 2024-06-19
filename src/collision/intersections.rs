use super::{
    plane::Plane,
    ray::Ray,
    root_finding::solve_quadratic,
    shapes::{cylinder::Cylinder, Cuboid, Ellipsoid, Sphere, Triangle},
};
use approx::abs_diff_eq;
use glam::Vec3;

pub fn ray_intersect_plane(ray: Ray, plane: Plane) -> Option<Vec3> {
    let v = ray.direction.extend(0.0);
    let homogeneous = plane.homogeneous();
    let den = homogeneous.dot(v);
    if abs_diff_eq!(den, 0.0, epsilon = 1e-2) {
        // Line parallel to plane
        None
    } else {
        let s = ray.start.extend(1.0);
        let num = homogeneous.dot(s);
        let t = -(num / den);
        (t > 0.0).then(|| ray.point(t))
    }
}

pub fn ray_intersect_triangle(ray: Ray, triangle: Triangle) -> Option<Vec3> {
    let normal = triangle.normal();
    let triangle_plane = Plane::new(triangle.v1, normal);

    if let Some(point) = ray_intersect_plane(ray, triangle_plane) {
        // Calculate baricentric coordinates to check if it is inside the triangle
        if triangle.is_inside_triangle(point) {
            return Some(point);
        }
    }
    None
}

#[allow(clippy::similar_names, clippy::useless_let_if_seq)]
pub fn ray_intersect_cuboid(ray: Ray, cuboid: Cuboid) -> Option<Vec3> {
    let plane_x0 = Plane::new(Vec3::ZERO, Vec3::X);
    let plane_xs = Plane::new(cuboid.size, Vec3::NEG_X);
    let plane_y0 = Plane::new(Vec3::ZERO, Vec3::Y);
    let plane_ys = Plane::new(cuboid.size, Vec3::NEG_Y);
    let plane_z0 = Plane::new(Vec3::ZERO, Vec3::Z);
    let plane_zs = Plane::new(cuboid.size, Vec3::NEG_Z);
    let mut planes: Vec<Plane> = Vec::with_capacity(3);
    if ray.direction.x > 0.0 {
        planes.push(plane_x0);
    } else if ray.direction.x < 0.0 {
        planes.push(plane_xs);
    }
    if ray.direction.y > 0.0 {
        planes.push(plane_y0);
    } else if ray.direction.y < 0.0 {
        planes.push(plane_ys);
    }
    if ray.direction.z > 0.0 {
        planes.push(plane_z0);
    } else if ray.direction.z < 0.0 {
        planes.push(plane_zs);
    }
    for plane in planes {
        if let Some(point) = ray_intersect_plane(ray, plane) {
            if cuboid.contains(point) {
                return Some(point);
            }
        }
    }

    None
}

pub fn ray_intersect_sphere(ray: Ray, sphere: Sphere) -> Option<Vec3> {
    let delta = ray.start - sphere.center;
    let a = 1.0; // Should be 1.0
    let b = 2.0 * ray.direction.dot(delta);
    let c = sphere
        .radius
        .mul_add(-sphere.radius, delta.length_squared());
    let solutions = solve_quadratic(a, b, c);
    let t = solutions.into_iter().min_by(f32::total_cmp)?;
    if t <= 0.0 {
        return None;
    }
    Some(ray.point(t))
}

#[allow(clippy::similar_names, clippy::many_single_char_names)]
pub fn ray_intersect_ellipsoid(ray: Ray, ellipse: Ellipsoid) -> Option<Vec3> {
    let vx = ray.direction.x;
    let vy = ray.direction.y;
    let vz = ray.direction.z;
    let vx2 = vx * vx;
    let vy2 = vy * vy;
    let vz2 = vz * vz;

    let sx = ray.start.x;
    let sy = ray.start.y;
    let sz = ray.start.z;
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

    Some(ray.point(t))
}

#[allow(clippy::similar_names, clippy::many_single_char_names)]
pub fn ray_intersect_cylinder(ray: Ray, cylinder: Cylinder) -> Option<Vec3> {
    let m = cylinder.radius_x / cylinder.radius_y;
    let m2 = m * m;
    let r = cylinder.radius_x;
    let vx = ray.direction.x;
    let vy = ray.direction.y;
    let vx2 = vx * vx;
    let vy2 = vy * vy;

    let sx = ray.start.x;
    let sy = ray.start.y;
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
    let point = ray.point(t);
    if point.z < 0.0 || point.z > cylinder.height {
        return None;
    }
    Some(point)
}

pub fn sphere_intersect_sphere(s1: Sphere, s2: Sphere) -> bool {
    let delta = s1.center - s2.center;
    let distance_sqr = delta.length_squared();
    let radius_sum = s1.radius + s2.radius;
    distance_sqr <= radius_sum * radius_sum
}

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;

    use approx::assert_abs_diff_eq;
    use proptest::prop_compose;
    use proptest::proptest;

    use crate::collision::intersections::sphere_intersect_sphere;
    use crate::collision::shapes::Cuboid;
    use crate::collision::{
        plane::Plane,
        ray::Ray,
        shapes::{cylinder::Cylinder, Ellipsoid, Sphere, Triangle},
    };
    use crate::tests::any_normal;
    use crate::tests::any_vec3;

    use super::ray_intersect_cuboid;
    use super::ray_intersect_cylinder;
    use super::ray_intersect_ellipsoid;
    use super::ray_intersect_plane;
    use super::ray_intersect_sphere;
    use super::ray_intersect_triangle;

    prop_compose! {
        fn any_ray(range:RangeInclusive<f32>)
                    (start in any_vec3(range),
                    direction in any_normal())
                    -> Ray {

            Ray::new(start,direction)
        }
    }
    prop_compose! {
        fn any_plane(range:RangeInclusive<f32>)
                    (point in any_vec3(range),
                    normal in any_normal())
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

    const RANGE: RangeInclusive<f32> = -100.0..=100.0;
    proptest! {

        #[test]
        fn test_ray_intersect_plane(line in any_ray(RANGE), plane in any_plane(RANGE)){
            _ray_intersect_plane(line, plane);
        }
        #[test]
        fn test_ray_intersect_triangle(line in any_ray(RANGE), triangle in any_triangle(RANGE)){
            _intersect_triangle(line, triangle);

        }
        #[test]
        fn test_ray_intersect_cuboid(mut line in any_ray(RANGE), size in any_vec3(0.0..=100.0)){
            line.start = line.start.min(size * 1.1);
            _intersect_cuboid(line, Cuboid::new(size));

        }
        #[test]
        fn test_ray_intersect_sphere(mut line in any_ray(RANGE),center in any_vec3(RANGE), radius in 0.0..=100.0_f32){
            line.start = line.start.min(center + radius * 1.1);
            _intersect_sphere(line, Sphere::new(center,radius));

        }
        #[test]
        fn test_ray_intersect_ellipse(mut line in any_ray(RANGE), radius in any_vec3(0.0..=100.0_f32)){
            line.start = line.start.min(radius * 1.1);
            _intersect_ellipse(line, Ellipsoid::new(radius));

        }
        #[test]
        fn test_ray_intersect_cylinder(mut line in any_ray(RANGE), size in any_vec3(0.0..=100.0_f32)){
            line.start = line.start.min(size * 1.1);
            _intersect_cylinder(line, Cylinder::new(size.x,size.y,size.z));

        }

        #[test]
        fn test_sphere_intersect_sphere(c1 in any_vec3(RANGE),r1 in 0.1..=100.0_f32,c2 in any_vec3(RANGE),r2 in 0.1..=100.0_f32){
            _sphere_intersect_sphere(Sphere::new(c1, r1), Sphere::new(c2, r2));

        }


    }
    fn _sphere_intersect_sphere(s1: Sphere, s2: Sphere) {
        let distance = s1.center.distance(s2.center);
        if distance <= s1.radius + s2.radius {
            assert!(sphere_intersect_sphere(s1, s2));
        } else {
            assert!(!sphere_intersect_sphere(s1, s2));
        }
    }
    fn _ray_intersect_plane(ray: Ray, plane: Plane) {
        if let Some(point) = ray_intersect_plane(ray, plane) {
            assert_abs_diff_eq!(plane.signed_distance_to(point), 0.0, epsilon = 0.1);
            assert_abs_diff_eq!(ray.distance_to(point), 0.0, epsilon = 0.1);
            let opposite_ray = Ray::new(ray.start, -ray.direction);
            let intersect = ray_intersect_plane(opposite_ray, plane);
            assert!(intersect.is_none());
        }
    }
    fn _intersect_triangle(ray: Ray, triangle: Triangle) {
        if let Some(point) = ray_intersect_triangle(ray, triangle) {
            let plane = Plane::new(triangle.v1, triangle.normal());
            assert_abs_diff_eq!(plane.signed_distance_to(point), 0.0, epsilon = 1e-1);
            assert_abs_diff_eq!(ray.distance_to(point), 0.0, epsilon = 1e-1);
            assert!(triangle.is_inside_triangle(point));
            let opposite_ray = Ray::new(ray.start, -ray.direction);
            let intersect = ray_intersect_triangle(opposite_ray, triangle);
            assert!(intersect.is_none());
        }
    }
    fn _intersect_cuboid(ray: Ray, cuboid: Cuboid) {
        if let Some(point) = ray_intersect_cuboid(ray, cuboid) {
            assert_abs_diff_eq!(ray.distance_to(point), 0.0, epsilon = 1e-1);
            assert!(cuboid.contains(point));
        }
    }
    fn _intersect_sphere(ray: Ray, sphere: Sphere) {
        if let Some(point) = ray_intersect_sphere(ray, sphere) {
            assert_abs_diff_eq!(ray.distance_to(point), 0.0, epsilon = 1e-2);

            assert_abs_diff_eq!(point.distance(sphere.center), sphere.radius, epsilon = 0.05);

            let opposite_ray = Ray::new(ray.start, -ray.direction);
            let intersect = ray_intersect_sphere(opposite_ray, sphere);
            assert!(intersect.is_none());
        }
    }
    fn _intersect_ellipse(ray: Ray, ellipse: Ellipsoid) {
        if let Some(point) = ray_intersect_ellipsoid(ray, ellipse) {
            assert_abs_diff_eq!(ray.distance_to(point), 0.0, epsilon = 1e-2);
        }
    }
    fn _intersect_cylinder(ray: Ray, cylinder: Cylinder) {
        if let Some(point) = ray_intersect_cylinder(ray, cylinder) {
            assert_abs_diff_eq!(ray.distance_to(point), 0.0, epsilon = 1e-3);

            assert_abs_diff_eq!(cylinder.equation(point), 0.0, epsilon = 1e-3);
            assert!(point.z >= 0.0 && point.z <= cylinder.height);
        }
    }
}
