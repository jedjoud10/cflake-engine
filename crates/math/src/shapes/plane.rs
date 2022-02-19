use super::Shapeable;

// An infinite plane
#[derive(Default, Clone)]
pub struct Plane {
    pub distance: f32,
    pub normal: veclib::Vector3<f32>,
}

impl Shapeable for Plane {
    fn get_center(&self) -> veclib::Vector3<f32> {
        self.normal * self.distance
    }
}
