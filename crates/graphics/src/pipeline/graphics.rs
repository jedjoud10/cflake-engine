use crate::{
    BlendConfig, CompareOp, CompiledDescription, DepthConfig,
    Graphics, Primitive, RenderPass,
    ShaderModule, StencilConfig, StencilOp, StencilTest, Shader,
};
use std::{mem::transmute, sync::Arc, ffi::CStr};
use vulkan::{vk, Device};

// A vulkan GRAPHICS pipeline abstraction that will handle initialization / destruction for us manually
// This will abstract most of the initialization and pain staking work of pipelines

// This pipeline is only to be used with the Material system in the "rendering" crate
// By itself, it only contains only 2 dynamic state that we can change after building it,
// which is it's viewport size and scissor testing
pub struct GraphicsPipeline {
    // Raw Vulkan
    pipeline: vk::Pipeline,

    // Immutable data set at build time
    depth_config: DepthConfig,
    stencil_config: StencilConfig,
    blend_config: BlendConfig,
    primitive: Primitive,

    // Keep the shader modules alive
    shader: Shader,
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            Graphics::global().device().destroy_pipeline(self.pipeline);
        }
    }
}

// Initialization of the graphics pipeline
impl GraphicsPipeline {
    // Create a new pipeline with the specified configs
    pub unsafe fn new(
        depth_config: DepthConfig,
        stencil_config: StencilConfig,
        blend_config: BlendConfig,
        primitive: Primitive,
        render_pass: &RenderPass,
        shader: Shader,
    ) -> Self {
        let graphics = Graphics::global();
        let pipeline = unsafe {
            // Viewport state
            let viewport_state = Self::build_viewport_state();

            // Input assembly state
            let input_assembly_state =
                Self::build_input_assembly_state(&primitive);

            // Rasterization state
            let rasterization_state = Self::build_rasterization_state(
                &primitive,
                &depth_config,
            );

            // Color blend state
            let color_blend_state =
                Self::build_color_blend_state(&blend_config);

            // Depth stencil state
            let depth_stencil_state = Self::build_depth_stencil_state(
                &stencil_config,
                &depth_config,
            );

            // Vertex input state
            let vertex_input_state = Self::build_vertex_input_state();

            // Dynamic state
            let dynamic_state = Self::build_dynamic_state();

            // Pipeline layout
            let layout =
                Self::build_pipeline_layout(graphics.device());

            // Multisample state
            let multisample_state = Self::build_multisampling_state();

            // Pipeline stages create info
            let stages = Self::build_stages(&shader);

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
            let attachment = [*attachment];
                
            // FIXME
            let blend_state = *vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op_enable(false)
                .logic_op(vk::LogicOp::COPY)
                .attachments(&attachment);

            let size = graphics.swapchain().extent();
            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: size.w as f32,
                height: size.h as f32,
                min_depth: 0.01,
                max_depth: 1.0,
            };
    
            let scissor = vk::Rect2D {
                offset: vk::Offset2D::default(),
                extent: vk::Extent2D {
                    width: size.w,
                    height: size.h,
                },
            };
    
            let viewport_state = *vk::PipelineViewportStateCreateInfo::builder()
                .viewports(&[viewport])
                .scissors(&[scissor]);

            // Create info for the graphics pipeline
            let create_info =
                vk::GraphicsPipelineCreateInfo::builder()
                    .color_blend_state(&blend_state)
                    .depth_stencil_state(&depth_stencil_state)
                    .dynamic_state(&dynamic_state)
                    .input_assembly_state(&input_assembly_state)
                    .layout(layout)
                    .rasterization_state(&rasterization_state)
                    .multisample_state(&multisample_state)
                    .render_pass(render_pass.renderpass())
                    .viewport_state(&viewport_state)
                    .stages(&stages)
                    .subpass(0)
                    .vertex_input_state(&vertex_input_state);
            graphics.device().create_graphics_pipeline(*create_info)
        };

        Self {
            pipeline,
            depth_config,
            stencil_config,
            blend_config,
            primitive,
            shader,
        }
    }

    // Create the viewport state
    fn build_viewport_state() -> vk::PipelineViewportStateCreateInfo {
        let graphics = Graphics::global();
        let size = graphics.swapchain().extent();
        log::warn!("{size}");
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: size.w as f32,
            height: size.h as f32,
            min_depth: 0.01,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: vk::Extent2D {
                width: size.w,
                height: size.h,
            },
        };

        *vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&[viewport])
            .scissors(&[scissor])
    }

    // Create the rasterization state
    fn build_rasterization_state(
        primitive: &Primitive,
        depth_config: &DepthConfig,
    ) -> vk::PipelineRasterizationStateCreateInfo {
        let mut builder =
            vk::PipelineRasterizationStateCreateInfo::builder();
        builder = primitive.apply_rasterization_state(builder);
        builder = depth_config.apply_rasterization_state(builder);
        *builder
    }

    // Create the input assembly state
    fn build_input_assembly_state(
        primitive: &Primitive,
    ) -> vk::PipelineInputAssemblyStateCreateInfo {
        let mut builder =
            vk::PipelineInputAssemblyStateCreateInfo::builder();
        builder = primitive.apply_input_assembly_state(builder);
        *builder
    }

    // Create the depth stencil state
    fn build_depth_stencil_state(
        stencil_config: &StencilConfig,
        depth_config: &DepthConfig,
    ) -> vk::PipelineDepthStencilStateCreateInfo {
        let mut builder =
            vk::PipelineDepthStencilStateCreateInfo::builder();
        builder = depth_config.apply_depth_stencil_state(builder);
        builder = stencil_config.apply_depth_stencil_state(builder);
        *builder
    }

    // Create the color blend state from the materil
    fn build_color_blend_state(
        blend_config: &BlendConfig,
    ) -> vk::PipelineColorBlendStateCreateInfo {
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
        let attachment = [*attachment];
        
        *vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&attachment)
    }

    // Create the pipeline layout
    fn build_pipeline_layout(device: &Device) -> vk::PipelineLayout {
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .flags(vk::PipelineLayoutCreateFlags::empty());

        unsafe {
            device
                .raw()
                .create_pipeline_layout(&create_info, None)
                .unwrap()
        }
    }

    // Create the multi-sampling state (I hate anti-aliasing. Fuck you. Cope)
    fn build_multisampling_state(
    ) -> vk::PipelineMultisampleStateCreateInfo {
        vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_mask(&[])
            .min_sample_shading(1.0f32)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
            .build()
    }

    // Create the vertex input state
    // TODO: Actually write this
    fn build_vertex_input_state(
    ) -> vk::PipelineVertexInputStateCreateInfo {
        vk::PipelineVertexInputStateCreateInfo::default()
    }

    // Get the dynamic state that will be modified per frame
    fn build_dynamic_state() -> vk::PipelineDynamicStateCreateInfo {
        /*
        let dynamic =
            &[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        */
        let dynamic = &[];

        vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(dynamic)
            .build()
    }

    // Create the shader stage create info using the compiled module descriptions
    // TODO: Change this I don't like it
    fn build_stages(shader: &Shader) -> Vec<vk::PipelineShaderStageCreateInfo> {
        let descriptions = [
            shader.vertex().description(),
            shader.fragment().description()
        ];

        descriptions
            .into_iter()
            .map(|c| {
                let stage = match c.kind {
                    crate::ModuleKind::Vertex => {
                        vk::ShaderStageFlags::VERTEX
                    }
                    crate::ModuleKind::Fragment => {
                        vk::ShaderStageFlags::FRAGMENT
                    }
                    crate::ModuleKind::Compute => {
                        vk::ShaderStageFlags::COMPUTE
                    }
                };

                *vk::PipelineShaderStageCreateInfo::builder()
                    .name(c.entry)
                    .flags(c.flags)
                    .module(*c.module)
                    .stage(stage)
            })
            .collect::<Vec<_>>()
    }
}

// Others
impl GraphicsPipeline {
    // Get the underlying raw Vulkan pipeline
    pub fn raw(&self) -> vk::Pipeline {
        self.pipeline
    }

    // Get the depth config used when creating this pipeline
    pub fn depth_config(&self) -> DepthConfig {
        self.depth_config
    }

    // Get the stencil config used when creating this pipeline
    pub fn stencil_config(&self) -> StencilConfig {
        self.stencil_config
    }

    // Get the blend config used when creating this pipeline
    pub fn blend_config(&self) -> BlendConfig {
        self.blend_config
    }

    // Get the primitive config used when creating this pipeline
    pub fn primitive(&self) -> Primitive {
        self.primitive
    }
}
