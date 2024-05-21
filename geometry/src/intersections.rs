use glam::{Mat2, Vec2, Vec3};

use crate::{
    line::Line,
    plane::Plane,
    root_finding::{solve_quadratic, solve_quartic, QuadraticSolution, QuarticSolution},
    shapes::{Cuboid, Cylinder, Ellipsoid, Sphere, Torus, Triangle},
};

pub type Ray = Line;
impl Ray {
    #[must_use]
    pub fn intersect_plane(&self, plane: Plane) -> Option<Vec3> {
        let v = self.direction.extend(0.0);
        let homogeneous = plane.homogeneous();
        let den = homogeneous.dot(v);
        if den == 0.0 {
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
            let r = point - triangle.v1;
            let q1 = triangle.v2 - triangle.v1;
            let q2 = triangle.v3 - triangle.v1;
            let dot = q1.dot(q2);
            let coefficients = Mat2::from_cols(
                [q1.length_squared(), dot].into(),
                [dot, q2.length_squared()].into(),
            );
            let constants = Vec2::new(r.dot(q1), r.dot(q2));
            let weights = coefficients.inverse() * constants;
            if weights.x + weights.y <= 1.0 {
                return Some(point);
            }
        }
        None
    }
    #[must_use]
    #[allow(clippy::similar_names, clippy::useless_let_if_seq)]
    pub fn intersect_cuboid(&self, cuboid: Cuboid) -> Option<Vec3> {
        let min = cuboid.min;
        let max = cuboid.max;
        let invdir = self.direction.recip();

        let mut tmin;
        let mut tmax;
        if invdir.x >= 0.0 {
            tmin = min.x - self.start.x * invdir.x;
            tmax = max.x - self.start.x * invdir.x;
        } else {
            tmin = max.x - self.start.x * invdir.x;
            tmax = min.x - self.start.x * invdir.x;
        }
        let tminy;
        let tmaxz;
        if invdir.y >= 0.0 {
            tminy = min.y - self.start.y * invdir.y;
            tmaxz = max.y - self.start.y * invdir.y;
        } else {
            tminy = max.y - self.start.y * invdir.y;
            tmaxz = min.y - self.start.y * invdir.y;
        };
        if (tmin > tmaxz) || (tminy > tmax) {
            return None;
        }

        if tminy > tmin {
            tmin = tminy;
        }
        if tmaxz < tmax {
            tmax = tmaxz;
        }
        let tminz;
        let tmaxz;
        if invdir.y >= 0.0 {
            tminz = min.y - self.start.y * invdir.y;
            tmaxz = max.y - self.start.y * invdir.y;
        } else {
            tminz = max.y - self.start.y * invdir.y;
            tmaxz = min.y - self.start.y * invdir.y;
        };

        if (tmin > tmaxz) || (tminz > tmax) {
            return None;
        }

        if tminz > tmin {
            tmin = tminz;
        }
        if tmaxz < tmax {
            tmax = tmaxz;
        }
        if tmin > 0.0 {
            Some(self.point(tmin))
        } else {
            None
        }
    }
    #[must_use]
    pub fn intersect_sphere(&self, sphere: Sphere) -> Option<Vec3> {
        let delta = self.start;
        let a = self.direction.length_squared(); // Should be 1.0
        let b = 2.0 * self.direction.dot(delta);
        let c = delta.length_squared() - sphere.radius * sphere.radius;
        let solutions = solve_quadratic(a, b, c);
        let t = match solutions {
            QuadraticSolution::OneReal(t) | QuadraticSolution::TwoReal(t, _) => t,
            QuadraticSolution::TwoComplex(_, _) => return None,
        };
        if t <= 0.0 {
            return None;
        }
        Some(self.point(t))
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

        let m = ellipse.semiaxis_xy;
        let n = ellipse.semiaxis_xz;
        let m2 = m * m;
        let n2 = n * n;

        let a = vx2 + m2 * vy2 + n2 * vz2;
        let b = 2.0 * (sx * vx + m2 * sy * vy + n2 * sz * vz);
        let c = sx2 + m2 * sy2 + n2 * sz2;

        let solution = solve_quadratic(a, b, c);
        let t = match solution {
            QuadraticSolution::OneReal(t) | QuadraticSolution::TwoReal(t, _) => t,
            QuadraticSolution::TwoComplex(_, _) => return None,
        };
        if t <= 0.0 {
            return None;
        }
        Some(self.point(t))
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
        let solution = solve_quadratic(a, b, c);
        let t = match solution {
            QuadraticSolution::OneReal(t) | QuadraticSolution::TwoReal(t, _) => t,
            QuadraticSolution::TwoComplex(_, _) => return None,
        };
        if t <= 0.0 {
            return None;
        }
        let point = self.point(t);
        if point.z < 0.0 || point.z > cylinder.height {
            return None;
        }
        Some(self.point(t))
    }

    #[must_use]
    #[allow(clippy::similar_names, clippy::many_single_char_names)]

    pub fn intersect_radius(&self, torus: Torus) -> Option<Vec3> {
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
        let solution = solve_quartic(a, b, c, d, e);
        let t = match solution {
            QuarticSolution::FourReal(t1, t2, t3, t4) => [t1, t2, t3, t4]
                .into_iter()
                .filter(|t| *t >= 0.0)
                .min_by(f32::total_cmp),
            QuarticSolution::ThreeReal(t1, t2, t3) => [t1, t2, t3]
                .into_iter()
                .filter(|t| *t >= 0.0)
                .min_by(f32::total_cmp),
            QuarticSolution::TwoRealTwoComplex(t1, t2, _, _) => [t1, t2]
                .into_iter()
                .filter(|t| *t >= 0.0)
                .min_by(f32::total_cmp),
            QuarticSolution::TwoReal(t1, t2) => [t1, t2]
                .into_iter()
                .filter(|t| *t >= 0.0)
                .min_by(f32::total_cmp),
            QuarticSolution::OneRealTwoComplex(t, _, _) => {
                if t > 0.0 {
                    Some(t)
                } else {
                    None
                }
            }
            QuarticSolution::FourComplex(_, _, _, _) => None,
        };
        t.map(|t| self.point(t))
    }
}
