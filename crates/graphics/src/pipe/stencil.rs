use std::mem::transmute;
use vulkan::vk;
use crate::CompareOp;


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

// Stencil testing wrapper 
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StencilTest {
    pub front_op: StencilState,
    pub back_op: StencilState,
}

// Stencil config wrapper
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StencilConfig(pub Option<StencilTest>);

impl StencilConfig {
    pub fn apply_depth_stencil_state<'a>(&self, builder: vk::PipelineDepthStencilStateCreateInfoBuilder<'a>) -> vk::PipelineDepthStencilStateCreateInfoBuilder<'a> {
        if let Some(stencil_test) = self.0 {
            builder
                .front(unsafe { transmute(stencil_test.front_op) })
                .back(unsafe { transmute(stencil_test.back_op) })
        } else {
            builder
        }
    }
}