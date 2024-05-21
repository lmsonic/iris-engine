#![allow(clippy::module_name_repetitions)]
use glam::{Mat4, Vec4};

use crate::plane::Plane;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frustum {
    pub near: Plane,
    pub far: Plane,
    pub left: Plane,
    pub right: Plane,
    pub bottom: Plane,
    pub top: Plane,
}

impl Frustum {
    #[must_use]
    pub const fn new(
        near: Plane,
        far: Plane,
        left: Plane,
        right: Plane,
        bottom: Plane,
        top: Plane,
    ) -> Self {
        Self {
            near,
            far,
            left,
            right,
            bottom,
            top,
        }
    }

    #[must_use]
    pub fn from_matrix_gl(matrix: Mat4) -> Self {
        let r1 = matrix.row(0);
        let r2 = matrix.row(1);
        let r3 = matrix.row(2);
        let r4 = matrix.row(3);
        let near = Plane::from_vec4(r4 + r3);
        let far = Plane::from_vec4(r4 - r3);
        let left = Plane::from_vec4(r4 + r1);
        let right = Plane::from_vec4(r4 - r1);
        let bottom = Plane::from_vec4(r4 + r2);
        let top = Plane::from_vec4(r4 - r2);

        Self {
            near,
            far,
            left,
            right,
            bottom,
            top,
        }
    }
    #[must_use]
    pub fn from_matrix(matrix: Mat4) -> Self {
        let r1 = matrix.row(0);
        let r2 = matrix.row(1);
        let r3 = 2.0 * matrix.row(2) - matrix.row(3);
        let r4 = matrix.row(3);
        let near = Plane::from_vec4(r4 + r3);
        let far = Plane::from_vec4(r4 - r3);
        let left = Plane::from_vec4(r4 + r1);
        let right = Plane::from_vec4(r4 - r1);
        let bottom = Plane::from_vec4(r4 + r2);
        let top = Plane::from_vec4(r4 - r2);

        Self {
            near,
            far,
            left,
            right,
            bottom,
            top,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PerspectiveFrustum {
    pub fov_y: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl PerspectiveFrustum {
    #[must_use]
    pub const fn new(fov_y: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            fov_y,
            aspect_ratio,
            near,
            far,
        }
    }

    #[must_use]
    pub fn matrix_lh(&self) -> Mat4 {
        Mat4::perspective_lh(self.fov_y, self.aspect_ratio, self.near, self.far)
    }
    #[must_use]
    pub fn matrix_rh(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect_ratio, self.near, self.far)
    }
    #[must_use]
    pub fn matrix_gl(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov_y, self.aspect_ratio, self.near, self.far)
    }
    #[must_use]
    pub fn matrix_infinite_lh(&self) -> Mat4 {
        Mat4::perspective_infinite_lh(self.fov_y, self.aspect_ratio, self.near)
    }
    #[must_use]
    pub fn matrix_infinite_rh(&self) -> Mat4 {
        Mat4::perspective_infinite_rh(self.fov_y, self.aspect_ratio, self.near)
    }
    #[must_use]
    pub fn matrix_infinite_gl(&self) -> Mat4 {
        let f = 1.0 / f32::tan(0.5 * self.fov_y);
        Mat4::from_cols(
            Vec4::new(f / self.aspect_ratio, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, -1.0, -1.0),
            Vec4::new(0.0, 0.0, -2.0 * self.near, 0.0),
        )
    }
    #[must_use]
    pub fn matrix_infinite_reverse_lh(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_lh(self.fov_y, self.aspect_ratio, self.near)
    }
    #[must_use]
    pub fn matrix_infinite_reverse_rh(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_rh(self.fov_y, self.aspect_ratio, self.near)
    }
    #[must_use]
    pub fn matrix_infinite_reverse_gl(&self) -> Mat4 {
        let f = 1.0 / f32::tan(0.5 * self.fov_y);
        Mat4::from_cols(
            Vec4::new(f / self.aspect_ratio, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, -1.0),
            Vec4::new(0.0, 0.0, 2.0 * self.near, 0.0),
        )
    }
    #[must_use]
    pub fn focal_distance(&self) -> f32 {
        ((self.fov_y * 0.5).tan() * self.aspect_ratio).recip()
    }
}

#[must_use]
pub fn clip_projection_matrix(clip_plane: Plane, mut matrix: Mat4) -> Mat4 {
    let inverse = matrix.inverse();
    let clip_plane = clip_plane.homogeneous();
    // transform into clip space
    let proj_clip_plane = inverse.transpose() * clip_plane;
    // Corner opposite to clip plane
    let proj_corner = Vec4::new(
        proj_clip_plane.x.signum(),
        proj_clip_plane.y.signum(),
        1.0,
        1.0,
    );
    // transform into camera space
    let corner = inverse * proj_corner;
    let r4 = matrix.row(3);
    // find the scale factor u for the plane to force far_plane.dot(corner) == 0 : far plane to contain the corner
    let u = 2.0 * r4.dot(corner) / clip_plane.dot(corner);
    let new_r3 = u * clip_plane - r4;

    matrix.x_axis[2] = new_r3.x;
    matrix.y_axis[2] = new_r3.y;
    matrix.z_axis[2] = new_r3.z;
    matrix.w_axis[2] = new_r3.w;
    matrix
}

#[derive(Clone, Copy, Debug)]
pub struct OrthographicFrustum {
    pub size: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl OrthographicFrustum {
    #[must_use]
    pub const fn new(size: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            size,
            aspect_ratio,
            near,
            far,
        }
    }

    #[must_use]
    pub fn matrix_lh(&self) -> Mat4 {
        let half_width = self.size * 0.5;
        let half_height = self.size * 0.5 * self.aspect_ratio;

        Mat4::orthographic_lh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            self.near,
            self.far,
        )
    }
    #[must_use]
    pub fn matrix_rh(&self) -> Mat4 {
        let half_width = self.size * 0.5;
        let half_height = self.size * 0.5 * self.aspect_ratio;

        Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            self.near,
            self.far,
        )
    }
    #[must_use]
    pub fn matrix_gl(&self) -> Mat4 {
        let half_width = self.size * 0.5;
        let half_height = self.size * 0.5 * self.aspect_ratio;

        Mat4::orthographic_rh_gl(
            -half_width,
            half_width,
            -half_height,
            half_height,
            self.near,
            self.far,
        )
    }
}

#[cfg(test)]
mod tests {

    use approx::{assert_abs_diff_eq, assert_relative_eq};
    use glam::{Vec3, Vec4};

    use crate::{
        frustum::{clip_projection_matrix, PerspectiveFrustum},
        plane::Plane,
    };
    use proptest::prelude::*;

    use super::{Frustum, OrthographicFrustum};
    #[derive(Debug, Clone, Copy)]
    enum ProjectionType {
        LeftHanded,
        RightHanded,
        OpenGL,
    }
    fn projection_type_strategy() -> impl Strategy<Value = ProjectionType> {
        prop_oneof![
            Just(ProjectionType::LeftHanded),
            Just(ProjectionType::RightHanded),
            Just(ProjectionType::OpenGL),
        ]
    }

    proptest! {
    #[test]
    fn test_orthographic_frustum(
        size in 0.1..100.0_f32,
        aspect_ratio in 0.1..3.0_f32,
        near in 0.1..1000.0_f32,
        far in (0.1..1000.0_f32),
        projection in projection_type_strategy(),
    ) {
        prop_assume!(near < far);
        _test_orthographic_frustum(size, aspect_ratio, near, far, projection);
    }
    }
    fn _test_orthographic_frustum(
        size: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
        projection: ProjectionType,
    ) {
        let ortho = OrthographicFrustum::new(size, aspect_ratio, near, far);
        let frustum = match projection {
            ProjectionType::LeftHanded => Frustum::from_matrix(ortho.matrix_lh()),
            ProjectionType::RightHanded => Frustum::from_matrix(ortho.matrix_rh()),
            ProjectionType::OpenGL => Frustum::from_matrix_gl(ortho.matrix_gl()),
        };

        let half_size = ortho.size * 0.5;

        let top = Plane::new(
            Vec3::new(0.0, ortho.aspect_ratio * half_size, 0.0),
            Vec3::NEG_Y,
        );
        let bottom = Plane::new(
            Vec3::new(0.0, -ortho.aspect_ratio * half_size, 0.0),
            Vec3::Y,
        );
        let right = Plane::new(Vec3::new(half_size, 0.0, 0.0), Vec3::NEG_X);
        let left = Plane::new(Vec3::new(-half_size, 0.0, 0.0), Vec3::X);
        let handedness = match projection {
            ProjectionType::LeftHanded => 1.0,
            ProjectionType::RightHanded | ProjectionType::OpenGL => -1.0,
        };
        let near = Plane::new(
            Vec3::new(0.0, 0.0, handedness * ortho.near),
            handedness * Vec3::Z,
        );
        let far = Plane::new(
            Vec3::new(0.0, 0.0, handedness * ortho.far),
            handedness * Vec3::NEG_Z,
        );

        assert_abs_diff_eq!(frustum.top, top, epsilon = 0.001);
        assert_abs_diff_eq!(frustum.bottom, bottom, epsilon = 0.001);
        assert_abs_diff_eq!(frustum.right, right, epsilon = 0.001);
        assert_abs_diff_eq!(frustum.left, left, epsilon = 0.001);
        assert_abs_diff_eq!(frustum.near, near, epsilon = 0.001);
        assert_abs_diff_eq!(frustum.far, far, epsilon = 0.001);
    }
    proptest! {
        #[test]
        fn test_perspective_frustum(
            fov_y in f32::to_radians(10.0)..f32::to_radians(180.0),
            aspect_ratio in 0.1..3.0_f32,
            near in 0.1..100.0_f32,
            far in 0.1..100.0_f32,
            projection in projection_type_strategy(),
        ) {
            prop_assume!(near < far);
            _test_perspective_frustum(fov_y, aspect_ratio, near, far, projection);
        }
    }
    fn _test_perspective_frustum(
        fov_y: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
        projection: ProjectionType,
    ) {
        let persp = PerspectiveFrustum::new(fov_y, aspect_ratio, near, far);
        let frustum = match projection {
            ProjectionType::LeftHanded => Frustum::from_matrix(persp.matrix_lh()),
            ProjectionType::RightHanded => Frustum::from_matrix(persp.matrix_rh()),
            ProjectionType::OpenGL => Frustum::from_matrix_gl(persp.matrix_gl()),
        };
        let handedness = match projection {
            ProjectionType::LeftHanded => 1.0,
            ProjectionType::RightHanded | ProjectionType::OpenGL => -1.0,
        };
        let focal_distance = persp.focal_distance();
        println!("fov = {}, ratio = {aspect_ratio}, near = {near}, far = {far}, type = {projection:?}, focal = {focal_distance}",fov_y.to_degrees());

        let top = Plane::new(
            Vec3::ZERO,
            Vec3::new(0.0, -focal_distance, handedness * aspect_ratio.recip()),
        );
        let bottom = Plane::new(
            Vec3::ZERO,
            Vec3::new(0.0, focal_distance, handedness * aspect_ratio.recip()),
        );
        let right = Plane::new(Vec3::ZERO, Vec3::new(-focal_distance, 0.0, handedness));
        let left = Plane::new(Vec3::ZERO, Vec3::new(focal_distance, 0.0, handedness));

        let near = Plane::new(
            Vec3::new(0.0, 0.0, handedness * persp.near),
            handedness * Vec3::Z,
        );
        let far = Plane::new(
            Vec3::new(0.0, 0.0, handedness * persp.far),
            handedness * Vec3::NEG_Z,
        );
        assert_relative_eq!(frustum.top, top, max_relative = 0.99);
        assert_relative_eq!(frustum.bottom, bottom, max_relative = 0.99);
        assert_relative_eq!(frustum.right, right, max_relative = 0.99);
        assert_relative_eq!(frustum.left, left, max_relative = 0.99);
        assert_relative_eq!(frustum.near, near, max_relative = 0.99);
        assert_relative_eq!(frustum.far, far, max_relative = 0.99);
    }
    proptest! {
        #[test]
        fn test_perspective_infinite_frustum(
            fov_y in f32::to_radians(10.0)..f32::to_radians(180.0),
            aspect_ratio in 0.1..3.0_f32,
            near in 0.1..1000.0_f32,
            projection in projection_type_strategy(),
        ) {
            _test_perspective_infinite_frustum(fov_y, aspect_ratio, near, projection);
        }
    }

    fn _test_perspective_infinite_frustum(
        fov_y: f32,
        aspect_ratio: f32,
        near: f32,
        projection: ProjectionType,
    ) {
        let persp = PerspectiveFrustum::new(fov_y, aspect_ratio, near, 100.0);
        let frustum = match projection {
            ProjectionType::LeftHanded => Frustum::from_matrix(persp.matrix_infinite_lh()),
            ProjectionType::RightHanded => Frustum::from_matrix(persp.matrix_infinite_rh()),
            ProjectionType::OpenGL => Frustum::from_matrix_gl(persp.matrix_infinite_gl()),
        };
        let handedness = match projection {
            ProjectionType::LeftHanded => 1.0,
            ProjectionType::RightHanded | ProjectionType::OpenGL => -1.0,
        };
        let focal_distance = persp.focal_distance();
        println!("fov = {}, ratio = {aspect_ratio}, near = {near}, type = {projection:?}, focal = {focal_distance}",fov_y.to_degrees());

        let top = Plane::new(
            Vec3::ZERO,
            Vec3::new(0.0, -focal_distance, handedness * aspect_ratio.recip()),
        );
        let bottom = Plane::new(
            Vec3::ZERO,
            Vec3::new(0.0, focal_distance, handedness * aspect_ratio.recip()),
        );
        let right = Plane::new(Vec3::ZERO, Vec3::new(-focal_distance, 0.0, handedness));
        let left = Plane::new(Vec3::ZERO, Vec3::new(focal_distance, 0.0, handedness));
        let near = Plane::new(
            Vec3::new(0.0, 0.0, handedness * persp.near),
            handedness * Vec3::Z,
        );

        assert_relative_eq!(frustum.top, top, max_relative = 0.99);
        assert_relative_eq!(frustum.bottom, bottom, max_relative = 0.99);
        assert_relative_eq!(frustum.right, right, max_relative = 0.99);
        assert_relative_eq!(frustum.left, left, max_relative = 0.99);
        assert_relative_eq!(frustum.near, near, max_relative = 0.99);
    }

    proptest! {
        #[test]
        fn test_perspective_infinite_reverse_frustum(
            fov_y in f32::to_radians(10.0)..f32::to_radians(180.0),
            aspect_ratio in 0.1..3.0_f32,
            near in 0.1..1000.0_f32,
            projection in projection_type_strategy(),
        ) {
            _test_perspective_infinite_frustum(fov_y, aspect_ratio, near, projection);
        }
    }

    fn _test_perspective_infinite_reverse_frustum(
        fov_y: f32,
        aspect_ratio: f32,
        near: f32,
        projection: ProjectionType,
    ) {
        let persp = PerspectiveFrustum::new(fov_y, aspect_ratio, near, 100.0);
        let frustum = match projection {
            ProjectionType::LeftHanded => Frustum::from_matrix(persp.matrix_infinite_reverse_lh()),
            ProjectionType::RightHanded => Frustum::from_matrix(persp.matrix_infinite_reverse_rh()),
            ProjectionType::OpenGL => Frustum::from_matrix_gl(persp.matrix_infinite_reverse_gl()),
        };
        let handedness = match projection {
            ProjectionType::LeftHanded => 1.0,
            ProjectionType::RightHanded | ProjectionType::OpenGL => -1.0,
        };
        let focal_distance = persp.focal_distance();
        println!("fov = {}, ratio = {aspect_ratio}, near = {near}, type = {projection:?}, focal = {focal_distance}",fov_y.to_degrees());

        let top = Plane::new(
            Vec3::ZERO,
            Vec3::new(0.0, -focal_distance, handedness * aspect_ratio.recip()),
        );
        let bottom = Plane::new(
            Vec3::ZERO,
            Vec3::new(0.0, focal_distance, handedness * aspect_ratio.recip()),
        );
        let right = Plane::new(Vec3::ZERO, Vec3::new(-focal_distance, 0.0, handedness));
        let left = Plane::new(Vec3::ZERO, Vec3::new(focal_distance, 0.0, handedness));
        let near = Plane::new(
            Vec3::new(0.0, 0.0, handedness * persp.near),
            handedness * Vec3::Z,
        );

        assert_relative_eq!(frustum.top, top, max_relative = 0.99);
        assert_relative_eq!(frustum.bottom, bottom, max_relative = 0.99);
        assert_relative_eq!(frustum.right, right, max_relative = 0.99);
        assert_relative_eq!(frustum.left, left, max_relative = 0.99);
        assert_relative_eq!(frustum.near, near, max_relative = 0.99);
    }

    #[test]
    fn test() {
        let perspective = PerspectiveFrustum::new(f32::to_radians(70.0), 1.2, 0.1, 100.0);
        let matrix = clip_projection_matrix(
            Plane::from_vec4(Vec4::new(1.0, 0.0, -1.0, -1.0)),
            perspective.matrix_infinite_gl(),
        );
        let frustum = Frustum::from_matrix(matrix);
        let dot = frustum.far.normal.dot(frustum.right.normal);
        dbg!(dot);
        // Bad precision
        assert!(dot > 0.9);
    }
}
