// How we read/write from depth attachments used by the material
#[derive(Clone, Copy, PartialEq)]
pub struct DepthConfig {
    pub compare: wgpu::CompareFunction,
    pub write_enabled: bool,
    pub depth_bias_constant: i32,
    pub depth_bias_slope_scale: f32,
    pub depth_bias_clamp: f32,
}