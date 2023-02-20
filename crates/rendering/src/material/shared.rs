use bytemuck::{Pod, Zeroable};
use graphics::{Normalized, Texture2D, UniformBuffer, RGBA};

// Camera data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct CameraUniform {
    // Projection & inv projection matrix
    pub projection: vek::Vec4<vek::Vec4<f32>>,
    pub inverse_projection: vek::Vec4<vek::Vec4<f32>>,

    // View & inv view matrix
    pub view: vek::Vec4<vek::Vec4<f32>>,
    pub inverse_view: vek::Vec4<vek::Vec4<f32>>,

    // Position of the camera
    //pub test: vek::Vec4<vek::Vec4<f32>>,

    // Direction vectors
    /*
    pub position: vek::Vec4<f32>,
    pub forward: vek::Vec4<f32>,
    pub right: vek::Vec4<f32>,
    pub up: vek::Vec4<f32>,
    */
}

// Timing data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct TimingUniform {
    pub frame_count: u32,
    pub delta_time: f32,
    pub time_since_startup: f32,
}

// Scene data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct SceneUniform {
    pub ambient_color: vek::Rgb<f32>,
    pub sun_strength: f32,
    pub sun_size: f32,
}

// Type aliases for textures
pub type AlbedoTexel = RGBA<Normalized<u8>>;
pub type NormalTexel = RGBA<Normalized<u8>>;
pub type AlbedoMap = Texture2D<AlbedoTexel>;
pub type NormalMap = Texture2D<NormalTexel>;

// Type aliases for buffers
pub type CameraBuffer = UniformBuffer<CameraUniform>;
pub type TimingBuffer = UniformBuffer<TimingUniform>;
pub type SceneBuffer = UniformBuffer<SceneUniform>;
