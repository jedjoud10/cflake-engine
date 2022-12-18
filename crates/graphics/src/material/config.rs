use vulkan::vk;

// How rasterized triangles should be culled
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FaceCullMode {
    Front(bool),
    Back(bool),
}

// Depicts the exact primitives we will use to draw the VAOs
#[derive(Clone, Copy, PartialEq)]
pub enum PrimitiveMode {
    Triangles { cull: Option<FaceCullMode> },
    Lines { width: f32 },
    Points,
}

// Comparison operator that represents the raw Vulkan comparison modes
// Equivalent to vk::CompareOp
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Never = 0,
    Less,
    Equal,
    LessThanOrEquals,
    Greater,
    NotEqual,
    GreaterThanOrEquals,
    Always,
}

// Stencil operator that represents the raw Vulkan stencil operations
// Equivalent to vk::StencilOp
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StencilOp {
    Keep = 0,
    Zero,
    Replace,
    IncrementAndClamp,
    DecrementAndClamp,
    Invert,
    IncrementAndWrap,
    DecrementAndWrap
}

// Wrapper around vk::StencilState
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StencilState {
    pub fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub compare_op: CompareOp,
    pub compare_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

// Wrapper around depth bound testing
#[derive(Clone, Copy, PartialEq)]
pub struct DepthBounds {
    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}

// Wrapper around depth bias
#[derive(Clone, Copy, PartialEq)]
pub struct DepthBias {
    pub depth_bias_enable: bool,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_slope_factor: f32,
}

// How we read/write from depth attachments used by the material
pub struct DepthConfig {
    pub depth_write_enable: bool,
    pub depth_clamp_enable: bool,
    pub depth_test: Option<CompareOp>,
    pub depth_bias: Option<DepthBias>,
    pub depth_bounds: Option<DepthBounds>,
}

// Stencil testing wrapper 
pub struct StencilTest {
    pub front_op: StencilState,
    pub back_op: StencilState,
}

pub struct BlendConfig {

}