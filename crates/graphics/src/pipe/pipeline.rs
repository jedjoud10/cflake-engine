use std::mem::transmute;
use vulkan::{vk, Device};
use crate::{DepthConfig, CompareOp, StencilOp, Primitive, StencilTest, BlendConfig, StencilConfig};

// A vulkan pipeline abstraction that will handle initialization / destruction for us manually
// This will abstract most of the initialization and 

// Create the rasterization state from the material
pub fn build_rasterization_state(primitive: &Primitive, depth_config: &DepthConfig) -> vk::PipelineRasterizationStateCreateInfo {
    let mut builder = vk::PipelineRasterizationStateCreateInfo::builder();
    builder = primitive.apply_rasterization_state(builder);
    builder = depth_config.apply_rasterization_state(builder);
    *builder
}

// Create the input assembly state
pub fn build_input_assembly_state(primitive: &Primitive) -> vk::PipelineInputAssemblyStateCreateInfo {
    let mut builder = vk::PipelineInputAssemblyStateCreateInfo::builder();
    builder = primitive.apply_input_assembly_state(builder);
    *builder
}

// Create the depth stencil state from the material
pub fn build_depth_stencil_state(stencil_config: &StencilConfig, depth_config: &DepthConfig) -> vk::PipelineDepthStencilStateCreateInfo {
    let mut builder = vk::PipelineDepthStencilStateCreateInfo::builder();
    builder = depth_config.apply_depth_stencil_state(builder);
    builder = stencil_config.apply_depth_stencil_state(builder);
    *builder
}

// Create the color blend state from the materil
pub fn build_color_blend_state(blend_config: &BlendConfig) -> vk::PipelineColorBlendStateCreateInfo {
    *vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
}

// Create the pipeline layout for a specific material
pub fn build_pipeline_layout() -> vk::PipelineLayout {
    todo!()
}

// Create the vertex input state for this specific material
pub fn build_vertex_input_state() -> vk::PipelineVertexInputStateCreateInfo {
    todo!()
}

// Get the dynamic state that will be modified per frame
pub fn build_dynamic_state() -> vk::PipelineDynamicStateCreateInfo {
    let dynamic = &[
        vk::DynamicState::VIEWPORT,
        vk::DynamicState::SCISSOR,
    ];
    
    vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(dynamic)
        .build()
}

/*
// Create a pipeline for usage for a specific material
pub unsafe fn build_pipeline(device: &Device) -> vk::Pipeline {
    let primitive = M::primitive();
    let depth_config = M::depth_config();
    let stencil_config = M::stencil_config();
    let blend_config = M::blend_config();
        
    let input_assembly_state = build_input_assembly_state(&primitive);
    let rasterization_state = build_rasterization_state(&primitive, &depth_config);
    let color_blend_state = build_color_blend_state(&blend_config);
    let depth_stencil_state = build_depth_stencil_state(&stencil_config, &depth_config);
    let vertex_input_state = build_vertex_input_state();
    let dynamic_state = build_dynamic_state();
    let layout = build_pipeline_layout();

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

    device.create_graphics_pipeline(*create_info)
}
*/