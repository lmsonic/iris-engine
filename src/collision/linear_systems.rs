use approx::abs_diff_eq;
use glam::{Mat2, Mat3, Mat3A, Mat4, Vec2, Vec3, Vec3A};

#[inline]
pub fn solve_linear_system_2d(coefficients: Mat2, constants: Vec2) -> Option<Vec2> {
    let determinant = coefficients.determinant();
    if abs_diff_eq!(determinant, 0.0) {
        None
    } else {
        Some(coefficients.inverse() * constants)
    }
}

#[inline]
pub fn solve_linear_system_3d(coefficients: Mat3, constants: Vec3) -> Option<Vec3> {
    let determinant = coefficients.determinant();
    if abs_diff_eq!(determinant, 0.0) {
        None
    } else {
        Some(coefficients.inverse() * constants)
    }
}
pub fn scalar_triple_product(u: impl Into<Vec3A>, v: impl Into<Vec3A>, w: impl Into<Vec3A>) -> f32 {
    Mat3A::from_cols(u.into(), v.into(), w.into()).determinant()
}

#[inline]
/// If ORIENT2D(A, B, C) > 0, C lies to the left of the directed line AB. Equivalently,
/// the triangle ABC is oriented counterclockwise. When ORIENT2D(A, B, C) < 0, C
/// lies to the right of the directed line AB, and the triangle ABC is oriented clockwise.
/// When ORIENT2D(A, B, C) = 0, the three points are collinear. The actual value
/// returned by ORIENT2D(A, B, C) corresponds to twice the signed area of the triangle ABC
/// (positive if ABC is counterclockwise, otherwise negative)
pub fn orient_2d(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    Mat3::from_cols(a.extend(1.0), b.extend(1.0), c.extend(1.0)).determinant()
}

#[inline]
/// When ORIENT3D(A, B, C, D) < 0, D lies above the supporting plane of triangle ABC,
/// in the sense that ABC appears in counterclockwise order when viewed from D.
/// If ORIENT3D(A, B, C, D) > 0, D instead lies below the plane of ABC.
/// When ORIENT3D(A, B, C, D) = 0, the four points are coplanar. The value returned by
/// ORIENT3D(A, B, C, D) corresponds to six times the signed volume of the tetrahedron
/// formed by the four points.
pub fn orient_3d(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> f32 {
    Mat4::from_cols(a.extend(1.0), b.extend(1.0), c.extend(1.0), d.extend(1.0)).determinant()
}
#[inline]
/// Let the triangle ABC appear in counterclockwise order, as indicated by  ORIENT2D(A, B, C) > 0.
/// Then, when INCIRCLE2D(A, B, C, D) > 0, D lies inside the circle through the three points A, B, and C.
/// If instead INCIRCLE2D(A, B, C, D) < 0, D lies outside the circle.
///  When INCIRCLE2D(A, B, C, D) = 0, the four points are cocircular.
/// If ORIENT2D(A, B, C) < 0, the result is reversed.
pub fn in_circle_2d(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> f32 {
    Mat4::from_cols(
        a.extend(a.length_squared()).extend(1.0),
        b.extend(b.length_squared()).extend(1.0),
        c.extend(c.length_squared()).extend(1.0),
        d.extend(d.length_squared()).extend(1.0),
    )
    .determinant()
}

#[inline]
#[allow(clippy::many_single_char_names)]
/// Let the four points A, B, C, and D be oriented such that ORIENT3D(A, B, C, D) > 0.
/// Then, when INSPHERE(A, B, C, D, E) > 0, E lies inside the sphere through A, B, C, and D.
/// If instead INSPHERE(A, B, C, D, E) < 0, E lies outside the sphere.
/// When INSPHERE(A, B, C, D, E) = 0, the five points are cospherical.
pub fn in_sphere(a: Vec3, b: Vec3, c: Vec3, d: Vec3, e: Vec3) -> f32 {
    let a = a - e;
    let b = b - e;
    let c = c - e;
    let d = d - e;
    Mat4::from_cols(
        a.extend(a.length_squared()),
        b.extend(b.length_squared()),
        c.extend(c.length_squared()),
        d.extend(d.length_squared()),
    )
    .determinant()
}
