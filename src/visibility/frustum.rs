#![allow(clippy::module_name_repetitions)]

use glam::{Mat3A, Mat4, Vec3A};

use crate::collision::shapes::Plane;

use super::bounding_volume::{aabb::Aabb, bounding_sphere::BoundingSphere, obb::Obb};

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

    pub fn intersect_bounding_sphere(&self, bounding_sphere: BoundingSphere) -> bool {
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
    pub fn intersect_bounding_box(&self, bounding_box: Aabb) -> bool {
        let center = bounding_box.center.extend(1.0);
        let size: Vec3A = bounding_box.size.into();

        for plane in self.planes() {
            // TODO : Might not cull boxes with very big difference in axis sizes
            let normal: Vec3A = plane.normal.into();
            let effective_radius = (normal * size).abs().element_sum();

            let plane = plane.homogeneous();
            if plane.dot(center) < -effective_radius {
                return false;
            }
        }
        true
    }
    pub fn intersect_oriented_bounding_box(&self, bounding_box: Obb) -> bool {
        let center = bounding_box.center().extend(1.0);
        let size = bounding_box.size();
        let orientation = Mat3A::from_quat(bounding_box.rotation);
        let (r, s, t) = (
            orientation.x_axis * size.x,
            orientation.y_axis * size.y,
            orientation.z_axis * size.z,
        );

        for plane in self.planes() {
            // TODO : Might not cull boxes with very big difference in axis sizes
            let normal = plane.normal.into();
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

#[cfg(test)]
mod tests {

    use approx::assert_relative_eq;
    use glam::Vec4;

    use crate::{
        collision::shapes::Plane,
        renderer::camera::{OrthographicCamera, PerspectiveCamera},
        visibility::frustum::FrustumBuilder,
    };
    use proptest::prelude::*;

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
        fn orthographic_frustum(
            size in 0.1..100.0_f32,
            aspect_ratio in 0.1..3.0_f32,
            near in 0.1..1000.0_f32,
            far in (0.1..1000.0_f32),
            projection in projection_type_strategy(),
        ) {
            prop_assume!(near < far);
            _orthographic_frustum(size, aspect_ratio, near, far, projection);
        }
        #[test]
        fn perspective_frustum(
            fov_y in f32::to_radians(10.0)..f32::to_radians(180.0),
            aspect_ratio in 0.1..3.0_f32,
            near in 0.1..100.0_f32,
            far in 0.1..100.0_f32,
            projection in projection_type_strategy(),
        ) {
            prop_assume!(near < far);
            _perspective_frustum(fov_y, aspect_ratio, near, far, projection);
        }
        #[test]
        fn perspective_infinite_frustum(
            fov_y in f32::to_radians(10.0)..f32::to_radians(180.0),
            aspect_ratio in 0.1..3.0_f32,
            near in 0.1..1000.0_f32,
            projection in projection_type_strategy(),
        ) {
            _perspective_infinite_frustum(fov_y, aspect_ratio, near, projection);
        }
        #[test]
        fn perspective_infinite_reverse_frustum(
            fov_y in f32::to_radians(10.0)..f32::to_radians(180.0),
            aspect_ratio in 0.1..3.0_f32,
            near in 0.1..1000.0_f32,
            projection in projection_type_strategy(),
        ) {
            _perspective_infinite_reverse_frustum(fov_y, aspect_ratio, near, projection);
        }
    }
    fn _orthographic_frustum(
        size: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
        projection: ProjectionType,
    ) {
        let ortho = OrthographicCamera::new(size, aspect_ratio, near, far);
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
    fn _perspective_frustum(
        fov_y: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
        projection: ProjectionType,
    ) {
        let persp = PerspectiveCamera::new(fov_y, aspect_ratio, near, far);
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

    fn _perspective_infinite_frustum(
        fov_y: f32,
        aspect_ratio: f32,
        near: f32,
        projection: ProjectionType,
    ) {
        let persp = PerspectiveCamera::new(fov_y, aspect_ratio, near, 100.0);
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

    fn _perspective_infinite_reverse_frustum(
        fov_y: f32,
        aspect_ratio: f32,
        near: f32,
        projection: ProjectionType,
    ) {
        let persp = PerspectiveCamera::new(fov_y, aspect_ratio, near, 100.0);
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
    fn near_plane_clipping() {
        let perspective = PerspectiveCamera::new(f32::to_radians(70.0), 1.2, 0.1, 100.0);

        let clip_plane: Plane = Vec4::new(1.0, 0.0, -1.0, -1.0).into();
        let matrix = clip_plane.clip_projection_matrix(perspective.matrix_infinite_gl());
        let frustum = FrustumBuilder::new(matrix).build();

        assert_relative_eq!(frustum.near, clip_plane, max_relative = 0.99);
        if let Some(far) = frustum.far {
            assert_relative_eq!(far.normal, frustum.right.normal, max_relative = 0.99);
        }
    }
}
