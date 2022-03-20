use crate::basics::lights::{LightTransform, LightType};

use super::{RenderedModel, ShadowedModel};

// Settings that tell us how we should render the scene
pub struct RenderingSettings<'scene, 'object> {
    // Normal objects
    pub normal: &'scene [RenderedModel<'object>],
    // Shadowed objects
    pub shadowed: &'scene [ShadowedModel<'object>],

    // Should we recalculate the shadow map?
    pub redraw_shadows: bool,

    // All the light sources
    pub lights: &'scene [(&'object LightType, LightTransform<'object>)],
}

// Camera rendering settings
#[derive(Default)]
pub struct RenderingCamera {
    // Position and rotation
    pub position: vek::Vec3<f32>,
    pub rotation: vek::Quaternion<f32>,

    // View and projection matrices
    pub viewm: vek::Mat4<f32>,
    pub projm: vek::Mat4<f32>,
    pub projm_viewm: vek::Mat4<f32>,

    // Near-Far clip planes
    pub clip_planes: vek::Vec2<f32>,
}
