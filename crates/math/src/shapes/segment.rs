use super::Shapeable;

// A simple, finite line segment
#[derive(Clone)]
pub struct Segment {
    // The start and end points for this line segment
    pub start: veclib::Vector3<f32>,
    pub end: veclib::Vector3<f32>,
}

impl Shapeable for Segment {
    fn get_center(&self) -> veclib::Vector3<f32> {
        (self.start + self.end) / 2.0
    }
}