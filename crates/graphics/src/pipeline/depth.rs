use crate::CompareOp;
use std::mem::transmute;
use vulkan::vk;

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

impl DepthConfig {
    pub fn apply_rasterization_state<'a>(
        &self,
        builder: vk::PipelineRasterizationStateCreateInfoBuilder<'a>,
    ) -> vk::PipelineRasterizationStateCreateInfoBuilder<'a> {
        let DepthConfig {
            depth_clamp_enable,
            depth_bias,
            ..
        } = *self;

        if let Some(depth_bias) = depth_bias {
            builder
                .depth_bias_constant_factor(
                    depth_bias.bias_constant_factor,
                )
                .depth_bias_slope_factor(depth_bias.bias_slope_factor)
                .depth_bias_clamp(depth_bias.bias_clamp)
                .depth_bias_enable(true)
        } else {
            builder
        }
        .depth_clamp_enable(depth_clamp_enable)
    }

    pub fn apply_depth_stencil_state<'a>(
        &self,
        mut builder: vk::PipelineDepthStencilStateCreateInfoBuilder<
            'a,
        >,
    ) -> vk::PipelineDepthStencilStateCreateInfoBuilder<'a> {
        let DepthConfig {
            depth_write_enable,
            depth_test,
            depth_bounds,
            ..
        } = *self;

        builder = builder
            .depth_write_enable(depth_write_enable)
            .depth_test_enable(depth_test.is_some())
            .depth_bounds_test_enable(depth_bounds.is_some());

        builder = if let Some(depth_test) = depth_test {
            builder.depth_compare_op(unsafe {
                transmute::<CompareOp, vk::CompareOp>(depth_test)
            })
        } else {
            builder.depth_compare_op(vk::CompareOp::ALWAYS)
        };

        if let Some(depth_bounds) = depth_bounds {
            builder
                .min_depth_bounds(depth_bounds.min_depth_bounds)
                .max_depth_bounds(depth_bounds.max_depth_bounds)
        } else {
            builder
        }
    }
}
