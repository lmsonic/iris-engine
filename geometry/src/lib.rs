use glam::Vec3;

pub mod frustum;
pub mod intersections;
pub mod plane;
pub mod ray;
pub mod root_finding;
pub mod shapes;

#[must_use]
// Assumes normal vectors
pub fn reflect(direction: Vec3, normal: Vec3) -> Vec3 {
    2.0 * (normal.dot(direction)) * normal - direction
}

#[must_use]
// Assumes normal vectors
pub fn refract(direction: Vec3, normal: Vec3, in_index: f32, out_index: f32) -> Vec3 {
    let dot = normal.dot(direction);
    let ratio = in_index / out_index;
    let ratio2 = ratio * ratio;
    ratio.mul_add(dot, -(ratio2).mul_add(-dot.mul_add(-dot, 1.0), 1.0).sqrt()) * normal
        - ratio * direction
}

#[cfg(test)]
mod tests {

    use std::ops::RangeInclusive;

    use approx::assert_abs_diff_eq;
    use glam::Vec3;
    use proptest::prop_compose;
    use proptest::proptest;
    use proptest::strategy::Strategy;

    use crate::reflect;

    prop_compose! {
        fn any_vec3(range:RangeInclusive<f32>)
                    (x in range.clone(),y in range.clone(),z in range)
                    -> Vec3 {
            Vec3::new(x, y, z)
        }
    }
    prop_compose! {
        fn any_normal(range:RangeInclusive<f32>)
                    (n in any_vec3(range).prop_filter_map("normal needs to be able to be normalized",
                    glam::Vec3::try_normalize))
                    -> Vec3 {
            n
        }
    }
    proptest! {
        #[test]
        fn test_reflect(input in any_normal(-1.0..=1.0),normal in any_normal(-1.0..=1.0)){
            _test_reflect(input, normal);
        }
    }

    fn _test_reflect(input: Vec3, normal: Vec3) {
        let output = reflect(input, normal);
        let angle1 = normal.angle_between(input);
        let angle2 = normal.angle_between(output);
        assert_abs_diff_eq!(angle1, angle2, epsilon = 1e-3);
    }
}
