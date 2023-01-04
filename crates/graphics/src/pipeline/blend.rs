use std::mem::transmute;

use vulkan::vk;

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

impl AttachmentBlendConfig {
    pub fn apply_color_blend_attachment_state<'a>(
        &self,
        builder: vk::PipelineColorBlendAttachmentStateBuilder<'a>,
    ) -> vk::PipelineColorBlendAttachmentStateBuilder<'a> {
        let color_write_mask = vk::ColorComponentFlags::from_raw(
            self.color_write_mask.bits()
        );
        
        unsafe {
            builder
                .color_write_mask(color_write_mask)
                .blend_enable(true)
                .src_color_blend_factor(transmute(self.src_color_blend_factor))
                .dst_color_blend_factor(transmute(self.dst_color_blend_factor))
                .color_blend_op(transmute(self.color_blend_op))
                .src_alpha_blend_factor(transmute(self.src_alpha_blend_factor))
                .dst_alpha_blend_factor(transmute(self.dstc_alpha_blend_factor))
                .alpha_blend_op(transmute(self.alpha_blend_op))
        }
    }

    pub fn apply_default_color_blend_attachment_state<'a>(
        builder: vk::PipelineColorBlendAttachmentStateBuilder<'a>,
    ) -> vk::PipelineColorBlendAttachmentStateBuilder<'a> {
        let color_write_mask = vk::ColorComponentFlags::from_raw(
            ColorComponentFlags::all().bits()
        );

        builder
            .color_write_mask(color_write_mask)
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ZERO)
            .dst_color_blend_factor(vk::BlendFactor::ONE)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ZERO)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE)
            .alpha_blend_op(vk::BlendOp::ADD)
    }
}

// How we deal with blending for the color attachments
#[derive(Clone, Copy)]
pub struct BlendConfig {
    pub logic_operation: Option<LogicOp>,
    pub attachments: Option<[AttachmentBlendConfig; 1]>,
}

impl BlendConfig {
    /*
    pub fn apply_color_blend_state<'a>(
        &'a self,
        mut builder: vk::PipelineColorBlendStateCreateInfoBuilder<'a>,
    ) -> vk::PipelineColorBlendStateCreateInfoBuilder<'a> {
        builder = builder
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&[]);

        // Color blend state attachment 0
        let mut attachment_builder = vk::PipelineColorBlendAttachmentState::builder();
        let attachments = self.attachments.map(|attachment| {
            attachment_builder = attachment[0].apply_color_blend_attachment_state(attachment_builder);
            [*attachment_builder]
        });

        // Apply the color blend attachments to the state
        if let Some(attachments) = attachments.as_ref() {
            builder = builder.attachments(attachments);
        } else {
            builder = builder.attachments(&[]);
        }
        builder
    }
    */
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
