use super::Shapeable;

// A simple 3D cube
#[derive(Clone)]
pub struct Cube {
    pub center: veclib::Vector3<f32>,

    // This is the full size of the cube, not the half-extent
    pub size: veclib::Vector3<f32>,
}

impl Shapeable for Cube {
    fn get_center(&self) -> veclib::Vector3<f32> {
        self.center
    }
}
