#![allow(clippy::module_name_repetitions)]

use glam::{Mat3, Mat4, Vec3, Vec4};

use crate::geometry::plane::Plane;

use super::bounding_volume::{BoundingSphere, Obb};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frustum {
    pub near: Plane,
    pub far: Option<Plane>,
    pub left: Plane,
    pub right: Plane,
    pub bottom: Plane,
    pub top: Plane,
}

impl Frustum {
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
            far: Some(far),
            left,
            right,
            bottom,
            top,
        }
    }

    pub fn planes(&self) -> Vec<Plane> {
        let mut planes = vec![self.left, self.right, self.bottom, self.top, self.near];
        if let Some(far) = self.far {
            planes.push(far);
        };
        planes
    }

    pub fn contains_bounding_sphere(&self, bounding_sphere: BoundingSphere) -> bool {
        let center = bounding_sphere.center.extend(1.0);
        for plane in self.planes() {
            let plane = plane.homogeneous();
            // Planes point inwards, so if the signed distance is less than -radius,
            // the sphere is out of frustum
            if plane.dot(center) < -bounding_sphere.radius {
                return false;
            }
        }
        true
    }
    pub fn contains_bounding_box(&self, bounding_box: impl Into<Obb>) -> bool {
        let bounding_box = bounding_box.into();
        self._contains_bounding_box(bounding_box)
    }
    fn _contains_bounding_box(&self, bounding_box: Obb) -> bool {
        let center = bounding_box.center().extend(1.0);
        let orientation = Mat3::from_quat(bounding_box.rotation);
        let (r, s, t) = (orientation.x_axis, orientation.y_axis, orientation.z_axis);

        for plane in self.planes() {
            // TODO : Might not cull boxes with very big difference in axis sizes
            let normal = plane.normal;
            let effective_radius =
                (r.dot(normal).abs() + s.dot(normal).abs() + t.dot(normal).abs()) * 0.5;

            let plane = plane.homogeneous();
            if plane.dot(center) < -effective_radius {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FrustumBuilder {
    reversed_z: bool,
    infinite: bool,
    is_opengl: bool,
    matrix: Mat4,
}

impl FrustumBuilder {
    pub fn new(matrix: Mat4) -> Self {
        Self {
            matrix,
            ..Default::default()
        }
    }
    pub fn reversed_z(&mut self) -> &mut Self {
        self.reversed_z = true;
        self
    }
    pub fn opengl(&mut self) -> &mut Self {
        self.is_opengl = true;
        self
    }
    pub fn infinite(&mut self) -> &mut Self {
        self.infinite = true;
        self
    }

    pub fn build(&mut self) -> Frustum {
        let matrix = self.matrix;
        let r1 = matrix.row(0);
        let r2 = matrix.row(1);
        let mut r3 = if self.is_opengl {
            matrix.row(2)
        } else {
            2.0 * matrix.row(2) - matrix.row(3)
        };
        if self.reversed_z {
            r3 = -r3;
        }
        let r4 = matrix.row(3);
        let near = (r4 + r3).into();
        let left = (r4 + r1).into();
        let right = (r4 - r1).into();
        let bottom = (r4 + r2).into();
        let top = (r4 - r2).into();
        let far = if self.infinite {
            None
        } else {
            Some((r4 - r3).into())
        };

        Frustum {
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
    pub const fn new(fov_y: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            fov_y,
            aspect_ratio,
            near,
            far,
        }
    }

    pub fn matrix_lh(&self) -> Mat4 {
        Mat4::perspective_lh(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    pub fn matrix_rh(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    pub fn matrix_gl(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    pub fn matrix_infinite_lh(&self) -> Mat4 {
        Mat4::perspective_infinite_lh(self.fov_y, self.aspect_ratio, self.near)
    }

    pub fn matrix_infinite_rh(&self) -> Mat4 {
        Mat4::perspective_infinite_rh(self.fov_y, self.aspect_ratio, self.near)
    }

    pub fn matrix_infinite_gl(&self) -> Mat4 {
        let f = 1.0 / f32::tan(0.5 * self.fov_y);
        Mat4::from_cols(
            Vec4::new(f / self.aspect_ratio, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, -1.0, -1.0),
            Vec4::new(0.0, 0.0, -2.0 * self.near, 0.0),
        )
    }

    pub fn matrix_infinite_reverse_lh(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_lh(self.fov_y, self.aspect_ratio, self.near)
    }

    pub fn matrix_infinite_reverse_rh(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_rh(self.fov_y, self.aspect_ratio, self.near)
    }

    pub fn matrix_infinite_reverse_gl(&self) -> Mat4 {
        let f = 1.0 / f32::tan(0.5 * self.fov_y);
        Mat4::from_cols(
            Vec4::new(f / self.aspect_ratio, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, -1.0),
            Vec4::new(0.0, 0.0, 2.0 * self.near, 0.0),
        )
    }

    pub fn focal_distance(&self) -> f32 {
        ((self.fov_y * 0.5).tan() * self.aspect_ratio).recip()
    }

    pub fn frustum(&self, is_left_handed: bool, infinite: bool) -> Frustum {
        let focal_distance = self.focal_distance();
        let aspect_ratio = self.aspect_ratio;
        let handedness = if is_left_handed { 1.0 } else { -1.0 };
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
            Vec3::new(0.0, 0.0, handedness * self.near),
            handedness * Vec3::Z,
        );
        let far = if infinite {
            None
        } else {
            Some(Plane::new(
                Vec3::new(0.0, 0.0, handedness * self.far),
                handedness * Vec3::NEG_Z,
            ))
        };
        Frustum {
            near,
            far,
            left,
            right,
            bottom,
            top,
        }
    }
}

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
    pub const fn new(size: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            size,
            aspect_ratio,
            near,
            far,
        }
    }

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

    pub fn frustum(&self, is_left_handed: bool) -> Frustum {
        let half_size = self.size * 0.5;

        let top = Plane::new(
            Vec3::new(0.0, self.aspect_ratio * half_size, 0.0),
            Vec3::NEG_Y,
        );
        let bottom = Plane::new(Vec3::new(0.0, -self.aspect_ratio * half_size, 0.0), Vec3::Y);
        let right = Plane::new(Vec3::new(half_size, 0.0, 0.0), Vec3::NEG_X);
        let left = Plane::new(Vec3::new(-half_size, 0.0, 0.0), Vec3::X);
        let handedness = if is_left_handed { 1.0 } else { -1.0 };

        let near = Plane::new(
            Vec3::new(0.0, 0.0, handedness * self.near),
            handedness * Vec3::Z,
        );
        let far = Plane::new(
            Vec3::new(0.0, 0.0, handedness * self.far),
            handedness * Vec3::NEG_Z,
        );
        Frustum::new(near, far, left, right, bottom, top)
    }
}

#[cfg(test)]
mod tests {

    use approx::assert_relative_eq;
    use glam::Vec4;

    use crate::visibility::frustum::{clip_projection_matrix, FrustumBuilder, PerspectiveFrustum};
    use proptest::prelude::*;

    use super::OrthographicFrustum;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        #[test]
        fn test_perspective_infinite_frustum(
            fov_y in f32::to_radians(10.0)..f32::to_radians(180.0),
            aspect_ratio in 0.1..3.0_f32,
            near in 0.1..1000.0_f32,
            projection in projection_type_strategy(),
        ) {
            _test_perspective_infinite_frustum(fov_y, aspect_ratio, near, projection);
        }
        #[test]
        fn test_perspective_infinite_reverse_frustum(
            fov_y in f32::to_radians(10.0)..f32::to_radians(180.0),
            aspect_ratio in 0.1..3.0_f32,
            near in 0.1..1000.0_f32,
            projection in projection_type_strategy(),
        ) {
            _test_perspective_infinite_reverse_frustum(fov_y, aspect_ratio, near, projection);
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
        let frustum_from_matrix = match projection {
            ProjectionType::LeftHanded => FrustumBuilder::new(ortho.matrix_lh()).build(),
            ProjectionType::RightHanded => FrustumBuilder::new(ortho.matrix_rh()).build(),
            ProjectionType::OpenGL => FrustumBuilder::new(ortho.matrix_gl()).opengl().build(),
        };

        let frustum = ortho.frustum(projection == ProjectionType::LeftHanded);

        assert_relative_eq!(frustum_from_matrix.top, frustum.top, max_relative = 0.99);
        assert_relative_eq!(
            frustum_from_matrix.bottom,
            frustum.bottom,
            max_relative = 0.99
        );
        assert_relative_eq!(
            frustum_from_matrix.right,
            frustum.right,
            max_relative = 0.99
        );
        assert_relative_eq!(frustum_from_matrix.left, frustum.left, max_relative = 0.99);
        assert_relative_eq!(frustum_from_matrix.near, frustum.near, max_relative = 0.99);
        if let Some(far) = frustum_from_matrix.far {
            if let Some(other) = frustum.far {
                assert_relative_eq!(far, other, max_relative = 0.99);
            }
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
        let frustum_from_matrix = match projection {
            ProjectionType::LeftHanded => FrustumBuilder::new(persp.matrix_lh()).build(),
            ProjectionType::RightHanded => FrustumBuilder::new(persp.matrix_rh()).build(),
            ProjectionType::OpenGL => FrustumBuilder::new(persp.matrix_gl()).opengl().build(),
        };
        let frustum = persp.frustum(projection == ProjectionType::LeftHanded, false);
        assert_relative_eq!(frustum_from_matrix.top, frustum.top, max_relative = 0.99);
        assert_relative_eq!(
            frustum_from_matrix.bottom,
            frustum.bottom,
            max_relative = 0.99
        );
        assert_relative_eq!(
            frustum_from_matrix.right,
            frustum.right,
            max_relative = 0.99
        );
        assert_relative_eq!(frustum_from_matrix.left, frustum.left, max_relative = 0.99);
        assert_relative_eq!(frustum_from_matrix.near, frustum.near, max_relative = 0.99);
        if let Some(far) = frustum_from_matrix.far {
            if let Some(other) = frustum.far {
                assert_relative_eq!(far, other, max_relative = 0.99);
            }
        }
    }

    fn _test_perspective_infinite_frustum(
        fov_y: f32,
        aspect_ratio: f32,
        near: f32,
        projection: ProjectionType,
    ) {
        let persp = PerspectiveFrustum::new(fov_y, aspect_ratio, near, 100.0);
        let frustum_from_matrix = match projection {
            ProjectionType::LeftHanded => FrustumBuilder::new(persp.matrix_infinite_lh())
                .infinite()
                .build(),
            ProjectionType::RightHanded => FrustumBuilder::new(persp.matrix_infinite_rh())
                .infinite()
                .build(),
            ProjectionType::OpenGL => FrustumBuilder::new(persp.matrix_infinite_gl())
                .opengl()
                .infinite()
                .build(),
        };
        let frustum = persp.frustum(projection == ProjectionType::LeftHanded, true);
        assert_relative_eq!(frustum_from_matrix.top, frustum.top, max_relative = 0.99);
        assert_relative_eq!(
            frustum_from_matrix.bottom,
            frustum.bottom,
            max_relative = 0.99
        );
        assert_relative_eq!(
            frustum_from_matrix.right,
            frustum.right,
            max_relative = 0.99
        );
        assert_relative_eq!(frustum_from_matrix.left, frustum.left, max_relative = 0.99);
        assert_relative_eq!(frustum_from_matrix.near, frustum.near, max_relative = 0.99);
        assert!(frustum_from_matrix.far.is_none());
        assert!(frustum.far.is_none());
    }

    fn _test_perspective_infinite_reverse_frustum(
        fov_y: f32,
        aspect_ratio: f32,
        near: f32,
        projection: ProjectionType,
    ) {
        let persp = PerspectiveFrustum::new(fov_y, aspect_ratio, near, 100.0);
        let frustum_from_matrix = match projection {
            ProjectionType::LeftHanded => FrustumBuilder::new(persp.matrix_infinite_reverse_lh())
                .reversed_z()
                .infinite()
                .build(),
            ProjectionType::RightHanded => FrustumBuilder::new(persp.matrix_infinite_reverse_rh())
                .reversed_z()
                .infinite()
                .build(),
            ProjectionType::OpenGL => FrustumBuilder::new(persp.matrix_infinite_reverse_gl())
                .opengl()
                .reversed_z()
                .infinite()
                .build(),
        };
        let frustum = persp.frustum(projection == ProjectionType::LeftHanded, true);
        assert_relative_eq!(frustum_from_matrix.top, frustum.top, max_relative = 0.99);
        assert_relative_eq!(
            frustum_from_matrix.bottom,
            frustum.bottom,
            max_relative = 0.99
        );
        assert_relative_eq!(
            frustum_from_matrix.right,
            frustum.right,
            max_relative = 0.99
        );
        assert_relative_eq!(frustum_from_matrix.left, frustum.left, max_relative = 0.99);
        assert_relative_eq!(frustum_from_matrix.near, frustum.near, max_relative = 0.99);
        assert!(frustum_from_matrix.far.is_none());
        assert!(frustum.far.is_none());
    }

    #[test]
    fn test_near_plane_clipping() {
        let perspective = PerspectiveFrustum::new(f32::to_radians(70.0), 1.2, 0.1, 100.0);

        let clip_plane = Vec4::new(1.0, 0.0, -1.0, -1.0).into();
        let matrix = clip_projection_matrix(clip_plane, perspective.matrix_infinite_gl());
        let frustum = FrustumBuilder::new(matrix).build();

        assert_relative_eq!(frustum.near, clip_plane, max_relative = 0.99);
        if let Some(far) = frustum.far {
            assert_relative_eq!(far.normal, frustum.right.normal, max_relative = 0.99);
        }
    }
}
