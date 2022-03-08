use super::{RenderedModel, ShadowedModel};

// Settings that tell us how we should render the scene
pub struct RenderingSettings<'a, 'b> {
    // Normal objects
    pub normal: &'a [RenderedModel<'b>],
    // Shadowed objects
    pub shadowed: &'a [ShadowedModel<'b>],
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
