use crate::pipeline::Pipeline;
use enum_as_inner::EnumAsInner;

// A light type
#[derive(EnumAsInner)]
pub enum LightSourceType {
    Directional(veclib::Quaternion<f32>),
}

// Main struct that contains some shared data about a light source
pub struct LightSource {
    // Main
    pub _type: LightSourceType,
    pub strength: f32,
    pub color: veclib::Vector3<f32>,
}

impl Default for LightSource {
    fn default() -> Self {
        Self {
            _type: LightSourceType::Directional(veclib::Quaternion::IDENTITY),
            strength: 1.0,
            color: veclib::Vector3::ONE,
        }
    }
}
