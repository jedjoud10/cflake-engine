use crate::CompareOp;
use std::mem::transmute;
use crate::vulkan::vk;

// Wrapper around depth bound testing
#[derive(Clone, Copy, PartialEq)]
pub struct DepthBounds {
    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}

// Wrapper around depth bias
#[derive(Clone, Copy, PartialEq)]
pub struct DepthBias {
    pub bias_clamp: f32,
    pub bias_constant_factor: f32,
    pub bias_slope_factor: f32,
}

// How we read/write from depth attachments used by the material
#[derive(Clone, Copy, PartialEq)]
pub struct DepthConfig {
    pub depth_write_enable: bool,
    pub depth_clamp_enable: bool,
    pub depth_test: Option<CompareOp>,
    pub depth_bias: Option<DepthBias>,
    pub depth_bounds: Option<DepthBounds>,
}