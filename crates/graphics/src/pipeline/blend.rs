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
        const R = 1 << 1;
        const G = 1 << 2;
        const B = 1 << 3;
        const A = 1 << 4;
    }
}

// Blend config for a single attachment
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
    pub logic_operation: Option<LogicOp>,
    pub attachments: [Option<AttachmentBlendConfig>; 1],
}

/*
            let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::R |
                vk::ColorComponentFlags::G |
                vk::ColorComponentFlags::B |
                vk::ColorComponentFlags::A)
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);
*/