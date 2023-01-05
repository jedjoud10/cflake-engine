use crate::{
    BlendConfig, DepthConfig, Graphics, Primitive, RenderPass,
    Shader, ShaderModule, StencilConfig, ColorLayout, DepthStencilLayout, PipelineInitializationError, AttachmentBlendConfig,
};

use vulkan::vk;
use winit::platform::unix::x11::ffi::XK_Left;

// A vulkan GRAPHICS pipeline abstraction that will handle initialization / destruction for us manually
// This will abstract most of the initialization and pain staking work of pipelines

// This pipeline is only to be used with the Material system in the "rendering" crate
// By itself, it only contains only 2 dynamic state that we can change after building it,
// which is it's viewport size and scissor testing
pub struct GraphicsPipeline {
    // Raw Vulkan
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,

    // Immutable data set at build time
    depth_config: DepthConfig,
    stencil_config: StencilConfig,
    blend_config: BlendConfig,
    primitive: Primitive,

    // Keep the shader modules alive
    shader: Shader,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.graphics.device().destroy_pipeline_layout(self.layout);
            self.graphics.device().destroy_pipeline(self.pipeline);
        }
    }
}

// Initialization of the graphics pipeline
impl GraphicsPipeline {
    // Create a new pipeline with the specified configs
    pub fn new<C: ColorLayout, DS: DepthStencilLayout>(
        graphics: &Graphics,
        depth_config: DepthConfig,
        stencil_config: StencilConfig,
        blend_config: BlendConfig,
        primitive: Primitive,
        render_pass: &RenderPass<C, DS>,
        shader: Shader,
    ) -> Result<Self, PipelineInitializationError> {
        let (pipeline, layout) = unsafe {
            // Common values that we reuse
            let depth_config = &depth_config;
            let primitive = &primitive;
            let stencil_config = &stencil_config;
            let _blend_config = &blend_config;
            let shader = &shader;

            // Viewport state set to nothing since it's dynamic state
            let viewport_state =
                *vk::PipelineViewportStateCreateInfo::builder()
                    .viewports(&[vk::Viewport::default()])
                    .scissors(&[vk::Rect2D::default()]);

            // Input assembly state
            let mut builder =
                vk::PipelineInputAssemblyStateCreateInfo::builder();
            builder = primitive.apply_input_assembly_state(builder);
            let input_assembly_state = builder;

            // Rasterization state
            let mut builder =
                vk::PipelineRasterizationStateCreateInfo::builder();
            builder = primitive.apply_rasterization_state(builder);
            builder = depth_config.apply_rasterization_state(builder);
            let rasterization_state = builder;

            // Color blend state
            let mut color_blend_builder = 
                vk::PipelineColorBlendStateCreateInfo::builder()
                    .logic_op_enable(false)
                    .logic_op(vk::LogicOp::COPY)
                    .attachments(&[]);

            // Color blend state attachment 0
            let attachment_builder = blend_config.attachments.map(|attachment| {
                attachment[0].apply_color_blend_attachment_state(vk::PipelineColorBlendAttachmentState::builder())
            }).unwrap_or_else(|| AttachmentBlendConfig::apply_default_color_blend_attachment_state(
                vk::PipelineColorBlendAttachmentState::builder()
            ));
            
            // Apply the color blend attachments to the state
            let penis = [*attachment_builder];
            let cock = penis.as_ref();
            color_blend_builder = color_blend_builder.attachments(cock);
            let color_blend_state = color_blend_builder;


            // Depth stencil state
            let mut builder =
                vk::PipelineDepthStencilStateCreateInfo::builder();
            builder = depth_config.apply_depth_stencil_state(builder);
            builder =
                stencil_config.apply_depth_stencil_state(builder);
            let depth_stencil_state = builder;

            // Vertex input state
            let vertex_input_state =
                vk::PipelineVertexInputStateCreateInfo::default();

            // Dynamic state
            let dynamic = [
                vk::DynamicState::VIEWPORT,
                vk::DynamicState::SCISSOR,
            ];
            let dynamic_state =
                vk::PipelineDynamicStateCreateInfo::builder()
                    .dynamic_states(&dynamic);

            // Pipeline layout
            let layout = {
                let create_info = vk::PipelineLayoutCreateInfo::builder()
                    .flags(vk::PipelineLayoutCreateFlags::empty());
                graphics.device().create_pipeline_layout(*create_info)
            };

            // Multisample state
            let multisample_state =
                vk::PipelineMultisampleStateCreateInfo::builder()
                    .sample_shading_enable(false)
                    .rasterization_samples(
                        vk::SampleCountFlags::TYPE_1,
                    )
                    .sample_mask(&[])
                    .min_sample_shading(1.0f32)
                    .alpha_to_coverage_enable(false)
                    .alpha_to_one_enable(false);

            // Pipeline stages create info
            let stages = shader.descriptions()
                .into_iter()
                .map(|description| {
                    let stage = description.kind.into_shader_stage_flags();
                    *vk::PipelineShaderStageCreateInfo::builder()
                        .name(description.entry)
                        .flags(description.flags)
                        .module(*description.module)
                        .stage(stage)
                        .specialization_info(&description.constants.raw)
                })
                .collect::<Vec<_>>();

            // Create info for the graphics pipeline
            let create_info_builder =
                vk::GraphicsPipelineCreateInfo::builder()
                    .color_blend_state(&color_blend_state)
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
            let create_info = *create_info_builder;
            (graphics.device().create_graphics_pipeline(create_info), layout)
        };

        Ok(Self {
            pipeline,
            layout,
            depth_config,
            stencil_config,
            blend_config,
            primitive,
            shader,
            graphics: graphics.clone(),
        })
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
