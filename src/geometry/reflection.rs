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

    use approx::assert_abs_diff_eq;
    use glam::Vec3;
    use proptest::proptest;

    use crate::geometry::reflection::Vec3ReflectExt;
    use crate::tests::any_normal;

    proptest! {
        #[test]
        fn reflect(input in any_normal(),normal in any_normal()){
            _reflect(input, normal);
        }
    }

    fn _reflect(input: Vec3, normal: Vec3) {
        let output = input.reflect(normal);
        let angle1 = normal.angle_between(input);
        let angle2 = normal.angle_between(output);
        assert_abs_diff_eq!(angle1, angle2, epsilon = 1e-3);
    }
}
