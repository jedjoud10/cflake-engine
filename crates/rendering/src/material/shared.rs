use bytemuck::{Pod, Zeroable};

// Camera data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    /*
    pub projection: vek::Mat4<f32>,
    pub inverse_projection: vek::Mat4<f32>,
    pub view: vek::Mat4<f32>,
    pub inverse_view: vek::Mat4<f32>,
    */
    pub position: vek::Vec4<f32>,
    pub forward: vek::Vec4<f32>,
    pub right: vek::Vec4<f32>,
}

// Timing data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct TimingUniform {
    pub frame_count: u32,
    pub delta_time: f32,
    pub time_since_startup: f32,
}

// Scene data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct SceneUniform {
    pub ambient_color: vek::Rgb<f32>,
    pub sun_strength: f32,
    pub sun_size: f32,
}