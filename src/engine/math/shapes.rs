// The corners of a cube
pub const CUBE_CORNERS: [glam::Vec3; 8] = [
    glam::const_vec3!([0.0, 0.0, 0.0]),
    glam::const_vec3!([1.0, 0.0, 0.0]),
    glam::const_vec3!([1.0, 0.0, 1.0]),
    glam::const_vec3!([0.0, 0.0, 1.0]),
    glam::const_vec3!([0.0, 1.0, 0.0]),
    glam::const_vec3!([1.0, 1.0, 0.0]),
    glam::const_vec3!([1.0, 1.0, 1.0]),
    glam::const_vec3!([0.0, 1.0, 1.0]),
];

// An infinite plane
#[derive(Default, Clone, Copy)]
pub struct Plane {
    pub distance: f32,
    pub normal: glam::Vec3,
}
// A simple, finite line
#[derive(Default, Clone, Copy)]
pub struct Line {
    pub point: glam::Vec3,
    pub point2: glam::Vec3,
}
impl Line {
    // Construct a line from it's start position and dir
    pub fn dir_construct(start: glam::Vec3, dir: glam::Vec3) -> Self {
        Self {
            point: start,
            point2: start + dir,
        }
    }
    // Construct a line from two points
    pub fn construct(start: glam::Vec3, end: glam::Vec3) -> Self {
        Self { point: start, point2: end }
    }
}
