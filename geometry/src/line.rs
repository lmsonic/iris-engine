use glam::{Mat2, Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub start: Vec3,
    pub direction: Vec3,
    pub is_ray: bool,
}

impl Line {
    #[must_use]
    pub fn new(start: Vec3, direction: Vec3) -> Self {
        Self {
            start,
            direction: direction.normalize(),
            is_ray: false,
        }
    }
    #[must_use]
    pub fn ray(start: Vec3, direction: Vec3) -> Self {
        Self {
            start,
            direction: direction.normalize(),
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
        delta.reject_from(self.direction).length()
    }

    #[must_use]
    pub fn closest_point_to(&self, point: Vec3) -> Vec3 {
        let t = self.closest_t_to(point);
        self.point(t)
    }
    #[must_use]
    pub fn closest_t_to(&self, point: Vec3) -> f32 {
        (point - self.start).dot(self.direction) / self.direction.length_squared()
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

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;

    use approx::{abs_diff_eq, assert_relative_eq};
    use glam::Vec3;
    use proptest::{prop_compose, proptest, strategy::Strategy};

    use super::Line;
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
        fn any_line(range:RangeInclusive<f32>)
                    (start in any_vec3(range.clone()),
                        direction in any_normal(range))
                    -> Line {

            Line::new(start,direction)
        }
    }

    proptest! {
        #[test]
        fn test_closest_point(line in any_line(-100.0..=100.0),point in any_vec3(-100.0..=100.0)){
            _test_closest_point(line, point);
        }
        #[test]
        fn test_distance_to_point(line in any_line(-100.0..=100.0),point in any_vec3(-100.0..=100.0)){
            _test_distance_to_point(line, point);
        }
    }

    fn _test_closest_point(line: Line, point: Vec3) {
        // Tests if it is a local minumum
        let closest_t = line.closest_t_to(point);
        let distance_to_closest = (line.point(closest_t)).distance(point);
        let distance_to_before = (line.point(closest_t - 0.2)).distance(point);
        let distance_to_after = (line.point(closest_t + 0.2)).distance(point);
        assert!(distance_to_closest < distance_to_before);
        assert!(distance_to_closest < distance_to_after);
    }
    fn _test_distance_to_point(line: Line, point: Vec3) {
        // Tests if it is a local minumum
        let closest_point = line.closest_point_to(point);

        assert_relative_eq!(
            closest_point.distance(point),
            line.distance_to_point(point),
            max_relative = 0.99
        );
    }
}
