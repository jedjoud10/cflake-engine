// An infinite plane
#[derive(Default)]
pub struct Plane {
    pub distance: f32,
    pub normal: glam::Vec3,
}
// A simple, finite line
#[derive(Default)]
pub struct Line {
    pub point: glam::Vec3,
    pub point2: glam::Vec3,
}