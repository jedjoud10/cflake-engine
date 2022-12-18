use std::mem::transmute;

use vulkan::{vk, Device};
use crate::{Material, DepthConfig, CompareOp, StencilOp, PrimitiveMode};

// Get the cull mode direction from the primtive mode
fn build_front_face(primitive: &PrimitiveMode) -> vk::FrontFace {
    match primitive {
        crate::PrimitiveMode::Triangles { cull, wireframe } => cull.map(|mode| {
            let ccw = match mode {
                crate::FaceCullMode::Front(ccw) => ccw,
                crate::FaceCullMode::Back(ccw) => ccw,
            };
    
            if ccw {
                vk::FrontFace::COUNTER_CLOCKWISE
            } else {
                vk::FrontFace::CLOCKWISE
            }
        }).unwrap_or(vk::FrontFace::CLOCKWISE),
        crate::PrimitiveMode::Lines { width } => vk::FrontFace::CLOCKWISE,
        crate::PrimitiveMode::Points => vk::FrontFace::CLOCKWISE,
    }
}

// Get the cull mode flags from the primtive mode
fn build_cull_mode_flags(primitive: &PrimitiveMode) -> vk::CullModeFlags {
    match primitive {
        crate::PrimitiveMode::Triangles { cull, wireframe } => {
            cull.map(|face_cull_mode| {
                match face_cull_mode {
                    crate::FaceCullMode::Front(_) => vk::CullModeFlags::FRONT,
                    crate::FaceCullMode::Back(_) => vk::CullModeFlags::BACK,
                }
            }).unwrap_or(vk::CullModeFlags::NONE)
        },
        crate::PrimitiveMode::Lines { width } => vk::CullModeFlags::NONE,
        crate::PrimitiveMode::Points => vk::CullModeFlags::NONE,
    }
}

// Get the polygon mode from the primitive mode
fn build_polygon_mode(primitive: &PrimitiveMode) -> vk::PolygonMode {
    match primitive {
        crate::PrimitiveMode::Triangles { cull, wireframe } => {
            if *wireframe {
                vk::PolygonMode::LINE
            } else {
                vk::PolygonMode::FILL
            }
        },
        crate::PrimitiveMode::Lines { width } => vk::PolygonMode::FILL,
        crate::PrimitiveMode::Points => vk::PolygonMode::POINT,
    }
}

// Create the rasterization state from the material
pub unsafe fn build_rasterization_state<M: Material>() -> vk::PipelineRasterizationStateCreateInfo {
    let primitive = M::primitive_mode();
    let polygon_mode = build_polygon_mode(&primitive);

    let DepthConfig {
        depth_write_enable,
        depth_clamp_enable,
        depth_test,
        depth_bias,
        depth_bounds,
    } = M::depth_config();
    
    let mut builder = vk::PipelineRasterizationStateCreateInfo::builder()
        .polygon_mode(polygon_mode)
        .depth_clamp_enable(depth_clamp_enable)
        .depth_bias_enable(depth_bias.is_some());

    let mut builder = if let Some(depth_bias) = depth_bias {
        let cull_mode = build_cull_mode_flags(&primitive);
        let front_face = build_front_face(&primitive);    

        builder
            .front_face(front_face)
            .cull_mode(cull_mode)
    } else {
        builder
    };

    let mut builder = if let Some(depth_bias) = depth_bias {
        builder
            .depth_bias_constant_factor(depth_bias.bias_constant_factor)
            .depth_bias_slope_factor(depth_bias.bias_slope_factor)
            .depth_bias_clamp(depth_bias.bias_clamp)
    } else {
        builder
    };

    *builder
}

// Create the input assembly state
pub unsafe fn build_input_assembly_state<M: Material>() -> vk::PipelineInputAssemblyStateCreateInfo {
    let topology = match M::primitive_mode() {
        crate::PrimitiveMode::Triangles { cull, wireframe } => vk::PrimitiveTopology::TRIANGLE_LIST,
        crate::PrimitiveMode::Lines { width } => vk::PrimitiveTopology::LINE_LIST,
        crate::PrimitiveMode::Points => vk::PrimitiveTopology::POINT_LIST,
    };

    vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(topology)
        .primitive_restart_enable(false)
        .build()
}

// Create the depth stencil state from the material
pub unsafe fn build_depth_stencil_state<M: Material>() -> vk::PipelineDepthStencilStateCreateInfo {
    let DepthConfig {
        depth_write_enable,
        depth_clamp_enable,
        depth_test,
        depth_bounds,
        ..
    } = M::depth_config();
    let stencil_test = M::stencil_testing();

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

    let mut builder = if let Some(stencil_test) = stencil_test {
        builder
            .front(transmute(stencil_test.front_op))
            .back(transmute(stencil_test.back_op))
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

    *builder
}

// Create the color blend state from the materil
pub unsafe fn build_color_blend_state<M: Material>() -> vk::PipelineColorBlendStateCreateInfo {
    todo!()
}

// Create the pipeline layout for a specific material
pub unsafe fn build_pipeline_layout<M: Material>(device: &Device) -> vk::PipelineLayout {
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
        
    let input_assembly_state = build_input_assembly_state::<M>();
    let rasterization_state = build_rasterization_state::<M>();
    let color_blend_state = build_color_blend_state::<M>();
    let depth_stencil_state = build_depth_stencil_state ::<M>();
    let vertex_input_state = build_vertex_input_state::<M>();
    let dynamic_state = build_dynamic_state::<M>();
    let layout = build_pipeline_layout::<M>(device);

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