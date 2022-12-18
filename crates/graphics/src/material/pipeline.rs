use vulkan::{vk, Device};
use crate::Material;

// Create the rasterization state from the material
pub unsafe fn build_rasterization_state<M: Material>() -> vk::PipelineRasterizationStateCreateInfo {
    todo!()
}

// Create the input assembly state
pub unsafe fn build_input_assembly_state<M: Material>() -> vk::PipelineInputAssemblyStateCreateInfo {
    vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
        .build()
}

// Create the depth stencil state from the material
pub unsafe fn build_depth_stencil_state<M: Material>() -> vk::PipelineDepthStencilStateCreateInfo {
    todo!()
}

// Create the color blend state from the materil
pub unsafe fn build_color_blend_state<M: Material>() -> vk::PipelineColorBlendStateCreateInfo {
    todo!()
}


// Create the pipeline layout for a specific material
pub unsafe fn build_pipeline_layout<M: Material>(device: &Device) -> vk::PipelineLayout {
    todo!()
}

// Create a pipeline for usage for a specific material
pub unsafe fn build_pipeline<M: Material>(device: &Device) -> vk::Pipeline {
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&[])
        .vertex_attribute_descriptions(&[])
        .build();
        
    let input_assembly_state = build_input_assembly_state::<M>();
    let rasterization_state = build_rasterization_state::<M>();
    let color_blend_state = build_color_blend_state::<M>();
    let depth_stencil_state = build_depth_stencil_state ::<M>();
    

    let create_info = vk::GraphicsPipelineCreateInfo::builder()
        .color_blend_state(&color_blend_state)
        .depth_stencil_state(&depth_stencil_state)
        .dynamic_state(dynamic_state)
        .flags(flags)
        .input_assembly_state(input_assembly_state)
        .layout(layout)
        .rasterization_state(&rasterization_state)
        .render_pass(render_pass)
        .stages(stages)
        .subpass(subpass)
        .vertex_input_state(vertex_input_state)
        .viewport_state(viewport_state);

    device.create_graphics_pipeline(&create_info)
}