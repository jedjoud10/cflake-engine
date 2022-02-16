use super::Shapeable;

// A simple 3D cube
pub struct Cube {
    pub center: veclib::Vector3<f32>,
    pub size: veclib::Vector3<f32>,
}

impl Shapeable for Cube {}
