use glam::{Mat2, Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub start: Vec3,
    pub direction: Vec3,
    pub is_ray: bool,
}

impl Line {
    #[must_use]
    #[allow(clippy::self_named_constructors)]
    pub const fn line(start: Vec3, direction: Vec3) -> Self {
        Self {
            start,
            direction,
            is_ray: false,
        }
    }
    #[must_use]
    pub const fn ray(start: Vec3, direction: Vec3) -> Self {
        Self {
            start,
            direction,
            is_ray: true,
        }
    }
    #[must_use]
    pub fn point(&self, t: f32) -> Vec3 {
        self.start + t * self.direction
    }

    #[must_use]
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        let delta = point - self.start;
        let distance_sqr =
            delta.length_squared() - delta.project_onto(self.direction).length_squared();
        distance_sqr.sqrt()
    }

    #[must_use]
    pub fn closest_to_point_on_line(&self, point: Vec3) -> Vec3 {
        let delta = point - self.start;
        let t = delta.project_onto(self.direction).length();
        self.point(t)
    }

    #[must_use]
    pub fn distance_to_line(&self, other: Self) -> f32 {
        let v1_sqr = self.direction.length_squared();
        let v2_sqr = other.direction.length_squared();
        let dot = self.direction.dot(other.direction);
        let coefficient_matrix = Mat2::from_cols_array_2d(&[[v1_sqr, dot], [-dot, v2_sqr]]);

        let delta = other.start - self.start;
        let constants = Vec2::new(delta.dot(self.direction), delta.dot(other.direction));

        let determinant = coefficient_matrix.determinant();
        if determinant == 0.0 {
            // Parallel lines, any point will do
            self.distance_to_point(other.start)
        } else {
            // Skew lines
            let inverse = coefficient_matrix.inverse();
            let t = inverse * constants;
            let p1 = self.point(t.x);
            let p2 = other.point(t.y);

            p1.distance(p2)
        }
    }
}
