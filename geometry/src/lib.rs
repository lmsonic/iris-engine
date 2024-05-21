use glam::Vec3;

pub mod frustum;
pub mod intersections;
pub mod line;
pub mod plane;
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
    (ratio * dot - (1.0 - (ratio2) * (1.0 - dot * dot)).sqrt()) * normal - ratio * direction
}
