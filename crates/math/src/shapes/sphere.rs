use super::Shapeable;

// A simple sphere
#[derive(Clone)]
pub struct Sphere {
    pub center: veclib::Vector3<f32>,
    pub radius: f32,
}

impl Shapeable for Sphere {
    fn get_center(&self) -> veclib::Vector3<f32> {
        self.center
    }
}
