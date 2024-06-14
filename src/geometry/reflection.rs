use glam::Vec3;

pub trait Vec3ReflectExt {
    // Assumes normal vectors
    fn reflect(&self, normal: Self) -> Self;
}

impl Vec3ReflectExt for Vec3 {
    fn reflect(&self, normal: Self) -> Self {
        2.0 * (normal.dot(*self)) * normal - *self
    }
}
pub trait Vec3RefractExt {
    // Assumes normal vectors
    fn refract(&self, normal: Self, in_index: f32, out_index: f32) -> Self;
}
impl Vec3RefractExt for Vec3 {
    fn refract(&self, normal: Self, in_index: f32, out_index: f32) -> Self {
        let dot = normal.dot(*self);
        let ratio = in_index / out_index;
        let ratio2 = ratio * ratio;
        ratio.mul_add(dot, -(ratio2).mul_add(-dot.mul_add(-dot, 1.0), 1.0).sqrt()) * normal
            - ratio * *self
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

    use crate::geometry::reflection::Vec3ReflectExt;

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
                    Vec3::try_normalize))
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
        let output = input.reflect(normal);
        let angle1 = normal.angle_between(input);
        let angle2 = normal.angle_between(output);
        assert_abs_diff_eq!(angle1, angle2, epsilon = 1e-3);
    }
}
