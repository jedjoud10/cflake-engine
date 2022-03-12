use crate::basics::lights::{LightTransform, LightType};

use super::{RenderedModel, ShadowedModel};

// Settings that tell us how we should render the scene
pub struct RenderingSettings<'scene, 'object> {
    // Normal objects
    pub normal: &'scene [RenderedModel<'object>],
    // Shadowed objects
    pub shadowed: &'scene [ShadowedModel<'object>],

    // All the light sources
    pub lights: &'scene [(&'object LightType, LightTransform<'object>)],
}

// Camera rendering settings
#[derive(Default)]
pub struct RenderingCamera {
    // Position and rotation
    pub position: veclib::Vector3<f32>,
    pub rotation: veclib::Quaternion<f32>,

    // View and projection matrices
    pub viewm: veclib::Matrix4x4<f32>,
    pub projm: veclib::Matrix4x4<f32>,
    pub projm_viewm: veclib::Matrix4x4<f32>,

    // Near-Far clip planes
    pub clip_planes: veclib::Vector2<f32>,
}
