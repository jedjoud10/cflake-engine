use std::mem::transmute;

use vulkan::{vk, Device};
use crate::{Material, DepthConfig, CompareOp, StencilOp, Primitive, StencilTest, BlendConfig};

// Create the rasterization state from the material
pub unsafe fn build_rasterization_state(primitive: &Primitive, depth_config: &DepthConfig) -> vk::PipelineRasterizationStateCreateInfo {
    let mut builder = vk::PipelineRasterizationStateCreateInfo::builder();
    primitive.apply_rasterization_state(&mut builder);
    depth_config.apply_rasterization_state(&mut builder);
    *builder
}

// Create the input assembly state
pub fn build_input_assembly_state(primitive: &Primitive) -> vk::PipelineInputAssemblyStateCreateInfo {
    let mut builder = vk::PipelineInputAssemblyStateCreateInfo::builder();
    primitive.apply_input_assembly_state(&mut builder);
    *builder
}

// Create the depth stencil state from the material
pub unsafe fn build_depth_stencil_state(stencil_test: &Option<StencilTest>, depth_config: &DepthConfig) -> vk::PipelineDepthStencilStateCreateInfo {
    let DepthConfig {
        depth_write_enable,
        depth_clamp_enable,
        depth_test,
        depth_bounds,
        ..
    } = *depth_config;

    let mut builder = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_write_enable(depth_write_enable)
        .depth_test_enable(depth_test.is_some())
        .depth_bounds_test_enable(depth_bounds.is_some())
        .stencil_test_enable(stencil_test.is_some());

    let mut builder = if let Some(depth_test) = depth_test {
        builder
            .depth_compare_op(transmute::<CompareOp, vk::CompareOp>(depth_test))
    } else {
        builder
    };

    let mut builder = if let Some(depth_bounds) = depth_bounds {
        builder
            .min_depth_bounds(depth_bounds.min_depth_bounds)
            .max_depth_bounds(depth_bounds.max_depth_bounds)
    } else {
        builder
    };

    let mut builder = if let Some(stencil_test) = stencil_test {
        builder
            .front(transmute(stencil_test.front_op))
            .back(transmute(stencil_test.back_op))
    } else {
        builder
    };



    *builder
}

// Create the color blend state from the materil
pub unsafe fn build_color_blend_state(blend_config: &BlendConfig) -> vk::PipelineColorBlendStateCreateInfo {
    todo!()
}

// Create the pipeline layout for a specific material
pub unsafe fn build_pipeline_layout<M: Material>() -> vk::PipelineLayout {
    todo!()
}

// Create the vertex input state for this specific material
pub unsafe fn build_vertex_input_state<M: Material>() -> vk::PipelineVertexInputStateCreateInfo {
    todo!()
}

// Get the dynamic state that will be modified per frame
pub unsafe fn build_dynamic_state<M: Material>() -> vk::PipelineDynamicStateCreateInfo {
    let viewport = vk::DynamicState::VIEWPORT;
    
    vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&[viewport])
        .build()
}

// Create a pipeline for usage for a specific material
pub unsafe fn build_pipeline<M: Material>(device: &Device) -> vk::Pipeline {
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&[])
        .vertex_attribute_descriptions(&[])
        .build();

    let primitive = M::primitive_mode();
    let depth_config = M::depth_config();
    let stencil_test = M::stencil_testing();
    let blend_config = M::blend_config();

        
    let input_assembly_state = build_input_assembly_state(&primitive);
    let rasterization_state = build_rasterization_state(&primitive, &depth_config);
    let color_blend_state = build_color_blend_state(&blend_config);
    let depth_stencil_state = build_depth_stencil_state(&stencil_test, &depth_config);
    let vertex_input_state = build_vertex_input_state::<M>();
    let dynamic_state = build_dynamic_state::<M>();
    let layout = build_pipeline_layout::<M>();

    let create_info = vk::GraphicsPipelineCreateInfo::builder()
        .color_blend_state(&color_blend_state)
        .depth_stencil_state(&depth_stencil_state)
        .dynamic_state(&dynamic_state)
        .input_assembly_state(&input_assembly_state)
        .layout(layout)
        .rasterization_state(&rasterization_state)
        .render_pass(render_pass)
        .stages(stages)
        .subpass(0)
        .vertex_input_state(&vertex_input_state);

    device.create_graphics_pipeline(*create_info)
}