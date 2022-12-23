use std::{mem::transmute, sync::Arc};
use vulkan::{vk, Device};
use crate::{DepthConfig, CompareOp, StencilOp, Primitive, StencilTest, BlendConfig, StencilConfig, Graphics, RenderPass, Stages};

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

    // Keep the graphics API alive
    graphics: Graphics,
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.graphics.device().destroy_pipeline(self.pipeline);
        }
    }
}

impl GraphicsPipeline {
    // Create a new pipeline with the specified configs
    pub unsafe fn new(
        graphics: &Graphics,
        depth_config: DepthConfig,
        stencil_config: StencilConfig,
        blend_config: BlendConfig,
        primitive: Primitive,
        render_pass: &RenderPass,
    ) -> Self {
        let pipeline = unsafe {
            let input_assembly_state = Self::build_input_assembly_state(&primitive);
            let rasterization_state = Self::build_rasterization_state(&primitive, &depth_config);
            let color_blend_state = Self::build_color_blend_state(&blend_config);
            let depth_stencil_state = Self::build_depth_stencil_state(&stencil_config, &depth_config);
            let vertex_input_state = Self::build_vertex_input_state();
            let dynamic_state = Self::build_dynamic_state();
            let layout = Self::build_pipeline_layout(graphics.device());
    

            let create_info = vk::GraphicsPipelineCreateInfo::builder()
                .color_blend_state(&color_blend_state)
                .depth_stencil_state(&depth_stencil_state)
                .dynamic_state(&dynamic_state)
                .input_assembly_state(&input_assembly_state)
                .layout(layout)
                .rasterization_state(&rasterization_state)
                .render_pass(todo!())
                .stages(todo!())
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
            graphics: graphics.clone(),
        }
    }

    // Create the rasterization state
    fn build_rasterization_state(primitive: &Primitive, depth_config: &DepthConfig) -> vk::PipelineRasterizationStateCreateInfo {
        let mut builder = vk::PipelineRasterizationStateCreateInfo::builder();
        builder = primitive.apply_rasterization_state(builder);
        builder = depth_config.apply_rasterization_state(builder);
        *builder
    }

    // Create the input assembly state
    fn build_input_assembly_state(primitive: &Primitive) -> vk::PipelineInputAssemblyStateCreateInfo {
        let mut builder = vk::PipelineInputAssemblyStateCreateInfo::builder();
        builder = primitive.apply_input_assembly_state(builder);
        *builder
    }

    // Create the depth stencil state
    fn build_depth_stencil_state(stencil_config: &StencilConfig, depth_config: &DepthConfig) -> vk::PipelineDepthStencilStateCreateInfo {
        let mut builder = vk::PipelineDepthStencilStateCreateInfo::builder();
        builder = depth_config.apply_depth_stencil_state(builder);
        builder = stencil_config.apply_depth_stencil_state(builder);
        *builder
    }

    // Create the color blend state from the materil
    fn build_color_blend_state(blend_config: &BlendConfig) -> vk::PipelineColorBlendStateCreateInfo {
        *vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
    }

    // Create the pipeline layout
    fn build_pipeline_layout(device: &Device) -> vk::PipelineLayout {
        let create_info = vk::PipelineLayoutCreateInfo::builder();

        unsafe {
            device
                .raw()
                .create_pipeline_layout(&create_info, None).unwrap()
        }
    }

    // Create the vertex input state
    fn build_vertex_input_state() -> vk::PipelineVertexInputStateCreateInfo {
        todo!()
    }

    // Get the dynamic state that will be modified per frame
    fn build_dynamic_state() -> vk::PipelineDynamicStateCreateInfo {
        let dynamic = &[
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR,
        ];

        vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(dynamic)
            .build()
    }
}