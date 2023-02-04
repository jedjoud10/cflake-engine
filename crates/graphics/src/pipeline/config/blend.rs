use std::mem::transmute;

use crate::vulkan::vk;

use crate::LogicOp;

// Equivalent to vk::BlendFactor
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlendFactor {
    Zero = 0,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SrcAlphaSaturate,
    Src1Color,
    OneMinusSrc1Color,
    Src1Alpha,
    OneMinusSrc1Alpha,
}

// Equivalent to vk::BlendOp
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlendOp {
    Add = 0,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

// Equivalent to vk::ColorComponentFlags
bitflags::bitflags! {
    pub struct ColorComponentFlags: u32 {
        const R = 1 << 0;
        const G = 1 << 1;
        const B = 1 << 2;
        const A = 1 << 3;
    }
}

// Blend config for a single color attachment
#[derive(Clone, Copy)]
pub struct AttachmentBlendConfig {
    pub color_write_mask: ColorComponentFlags,
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op: BlendOp,
    pub src_alpha_blend_factor: BlendFactor,
    pub dstc_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op: BlendOp,
}

// How we deal with blending for the color attachments
#[derive(Clone, Copy)]
pub struct BlendConfig {
}