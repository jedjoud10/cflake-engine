use super::Shapeable;

// A simple sphere
pub struct Sphere {
    pub center: veclib::Vector3<f32>,
    pub radius: f32,
}

impl Shapeable for Sphere {}