use super::LightSourceType;

// A simple directional light
pub struct Directional {
    pub direction: veclib::Vector3<f32>,
}

impl Directional {
    // Create a new directional light source
    pub fn new(dir: veclib::Vector3<f32>) -> LightSourceType {
        LightSourceType::Directional(Self {
            direction: dir.normalized(),
        })
    }
}