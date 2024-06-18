use std::{f32::consts::TAU, ops::RangeInclusive};

use glam::{Quat, Vec3};
use proptest::{prop_compose, strategy::Strategy};

prop_compose! {
    pub fn any_vec3(range:RangeInclusive<f32>)
                (x in range.clone(),y in range.clone(),z in range)
                -> Vec3 {
        Vec3::new(x, y, z)
    }
}
const NORMAL_RANGE: RangeInclusive<f32> = -1.0..=1.0;
prop_compose! {
    pub  fn any_normal()
                (n in any_vec3(NORMAL_RANGE).prop_filter_map("normal needs to be able to be normalized",
                Vec3::try_normalize))
                -> Vec3 {
        n
    }
}
prop_compose! {
    pub  fn any_quat()
                (axis in any_vec3(NORMAL_RANGE).prop_filter_map("normal needs to be able to be normalized",
                Vec3::try_normalize),
                angle in -TAU..=TAU)
                -> Quat {
        Quat::from_axis_angle(axis, angle)
    }
}
