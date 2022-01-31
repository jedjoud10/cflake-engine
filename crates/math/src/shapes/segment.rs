use super::Shapeable;

// A simple, finite line segment
#[derive(Default, Clone, Copy)]
pub struct Segment {
    pub start: veclib::Vector3<f32>,
    pub end: veclib::Vector3<f32>,
}
impl Segment {
    // Construct a line from it's start position and direction
    pub fn dir_construct(start: veclib::Vector3<f32>, dir: veclib::Vector3<f32>) -> Self {
        Self {
            start,
            end: start + dir,
        }
    }
    // Construct a line from two points
    pub fn new(start: veclib::Vector3<f32>, end: veclib::Vector3<f32>) -> Self {
        Self { start, end }
    }
}

impl Shapeable for Segment {}