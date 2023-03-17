use bytemuck::{Pod, Zeroable};
use graphics::{Normalized, Texture2D, UniformBuffer, RGBA, SRGBA, RG};

// Camera data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct CameraUniform {
    // Projection & view matrix
    pub projection: vek::Vec4<vek::Vec4<f32>>,
    pub view: vek::Vec4<vek::Vec4<f32>>,

    // Position and direction vectors
    pub position: vek::Vec4<f32>,
    pub forward: vek::Vec4<f32>,
    pub right: vek::Vec4<f32>,
    pub up: vek::Vec4<f32>,
}

// Timing data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
// TODO: IMPLEMENT
pub struct TimingUniform {
    pub frame_count: u32,
    pub delta_time: f32,
    pub time_since_startup: f32,
}

// Scene data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct SceneUniform {
    // Sun related parameters
    pub sun_direction: vek::Vec4<f32>,
    pub sun_color: vek::Rgba<f32>,

    // Ambient color of the environment
    pub ambient_color_strength: f32,

    // Procedural sun parameters
    pub sun_circle_strength: f32,
    pub sun_circle_size: f32,
    pub sun_circle_fade: f32,
}

// Window/monitor data thw ill be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C, align(4))]
pub struct WindowUniform {
    pub width: u32,
    pub height: u32,
}

// Type aliases for texels
//pub type AlbedoTexel = SRGBA<Normalized<graphics::UBC1>>;
pub type AlbedoTexel = SRGBA<Normalized<u8>>;
pub type NormalTexel = RG<Normalized<u8>>;
pub type MaskTexel = RGBA<Normalized<u8>>;

// Type aliases for textures
pub type AlbedoMap = Texture2D<AlbedoTexel>;
pub type NormalMap = Texture2D<NormalTexel>;
pub type MaskMap = Texture2D<MaskTexel>;

// Type aliases for buffers
pub type CameraBuffer = UniformBuffer<CameraUniform>;
pub type TimingBuffer = UniformBuffer<TimingUniform>;
pub type SceneBuffer = UniformBuffer<SceneUniform>;
pub type WindowBuffer = UniformBuffer<WindowUniform>;
