use crate::basics::lights::{LightTransform, LightType};

use super::{RenderedModel, ShadowedModel};

// Settings that tell us how we should render the scene
pub struct RenderingSettings<'object> {
    // Normal objects
    pub normal: Vec<RenderedModel<'object>>,
    // Shadowed objects
    pub shadowed: Vec<ShadowedModel<'object>>,

    // Should we recalculate the shadow map?
    pub redraw_shadows: bool,

    // All the light sources
    pub lights: Vec<(&'object LightType, LightTransform<'object>)>,
}

// Camera rendering settings
#[derive(Default)]
pub struct RenderingCamera {
    // Position and rotation and forward vector
    pub position: vek::Vec3<f32>,
    pub rotation: vek::Quaternion<f32>,
    pub forward: vek::Vec3<f32>,

    // View and projection matrices
    pub view: vek::Mat4<f32>,
    pub proj: vek::Mat4<f32>,
    pub proj_view: vek::Mat4<f32>,

    // Near-Far clip planes
    pub clips: vek::Vec2<f32>,
}

// The first valid directional light in our world
pub(super) struct Sun {
    // Forward vector of the sun
    pub dir: vek::Vec3<f32>,

    // Color + Strength combined
    pub color: vek::Rgb<f32>,
}

impl Default for Sun {
    fn default() -> Self {
        Self { dir: Default::default(), color: Default::default() }
    }
}