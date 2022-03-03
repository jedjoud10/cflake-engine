use crate::{
    pipeline::Pipeline,
};
use enum_as_inner::EnumAsInner;

// A light type
#[derive(EnumAsInner)]
pub enum LightSourceType {
    Directional { quat: veclib::Quaternion<f32> },
}

// Main struct that contains some shared data about a light source
pub struct LightSource {
    // Main
    pub _type: LightSourceType,
    pub strength: f32,
    pub color: veclib::Vector3<f32>,
}

impl LightSource {
    // Create a new light source
    pub fn new(_type: LightSourceType) -> Self {
        Self {
            _type,
            color: veclib::Vector3::ONE,
            strength: 1.0,
        }
    }
    // Create this light source with a specififed strength
    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = strength;
        self
    }
    // Create this light source with a specified color
    pub fn with_color(mut self, color: veclib::Vector3<f32>) -> Self {
        self.color = color;
        self
    }
}
