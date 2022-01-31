use super::Shapeable;


// An infinite plane
#[derive(Default, Clone, Copy)]
pub struct Plane {
    pub distance: f32,
    pub normal: veclib::Vector3<f32>,
}

impl Shapeable for Plane {}