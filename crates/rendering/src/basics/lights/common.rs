use crate::{object::PipelineCollectionElement, pipeline::Pipeline};
use enum_as_inner::EnumAsInner;

// Light source parameters
pub struct LightParameters {
    pub strength: f32,
    pub color: veclib::Vector3<f32>,
}

impl Default for LightParameters {
    fn default() -> Self {
        Self {
            strength: 1.0,
            color: veclib::Vector3::ONE,
        }
    }
}

// Light transform
pub struct LightTransform<'object> {
    pub position: &'object veclib::Vector3<f32>,
    pub rotation: &'object veclib::Quaternion<f32>,
}

// A light type
#[derive(EnumAsInner)]
pub enum LightType {
    // Directional light, like the sun
    Directional { params: LightParameters },
    // Point light, like a lamp
    // TODO: Add support for point lights in shader
    Point { params: LightParameters, radius: f32 },
}
