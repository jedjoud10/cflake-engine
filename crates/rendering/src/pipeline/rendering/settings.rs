use super::{ShadowedModel, RenderedModel};


// Settings that tell us how we should render the scene
#[derive(Clone)]
pub struct RenderingSettings<'a, 'b> {
    // Normal objects
    pub normal: &'a [RenderedModel<'b>],
    // Shadowed objects
    pub shadowed: &'a [ShadowedModel<'b>],

    // Camera settings
    pub camera: RenderingCamera<'b>,
}

// Camera rendering settings
#[derive(Clone)]
pub struct RenderingCamera<'b> {
    // Position and rotation
    pub position: &'b veclib::Vector3<f32>,
    pub rotation: &'b veclib::Quaternion<f32>,

    // View and projection matrices
    pub viewm: &'b veclib::Matrix4x4<f32>,
    pub projm: &'b veclib::Matrix4x4<f32>,
    pub projm_viewm: &'b veclib::Matrix4x4<f32>,
    pub forward: &'b veclib::Vector3<f32>,

    // Near-Far clip planes
    pub clip_planes: &'b veclib::Vector2<f32>,
}