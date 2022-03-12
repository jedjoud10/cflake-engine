// A simple 3D cuboid
#[derive(Clone)]
pub struct Cuboid {
    pub center: veclib::Vector3<f32>,

    // This is the full size of the cuboid, not the half-extent
    pub size: veclib::Vector3<f32>,
}

impl Default for Cuboid {
    fn default() -> Self {
        Self { center: Default::default(), size: veclib::Vector3::ONE }
    }
}