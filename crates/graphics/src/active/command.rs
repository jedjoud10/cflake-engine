use crate::{
    BufferInfo, ColorLayout, ComputeShader, DepthStencilLayout,
    DrawIndexedIndirectBuffer, DrawIndirectBuffer, RenderPipeline,
    TriangleBuffer, UniformBuffer,
};
use std::{
    ops::{Bound, Range},
    sync::Arc,
};
use wgpu::{util::RenderEncoder, BindGroup};

// Keep track of all render commands that we call upon the render pass
// The whole reason I have to delegate this to a command type system is because
// the render pass requires the BindGroups to live longer than itself, and I couldn't make it work
pub(crate) enum RenderCommand<
    'a,
    C: ColorLayout,
    DS: DepthStencilLayout,
> {
    // Bind graphics pipeline
    BindPipeline(&'a RenderPipeline<C, DS>),

    // Set vertex buffer for Draw and DrawIndexed
    SetVertexBuffer {
        slot: u32,
        buffer: BufferInfo<'a>,
        start: Bound<u64>,
        end: Bound<u64>,
    },

    // Set index buffer for DrawIndexed
    SetIndexBuffer {
        buffer: &'a TriangleBuffer<u32>,
        start: Bound<u64>,
        end: Bound<u64>,
    },

    // Set bind group
    SetBindGroup(u32, Arc<BindGroup>),

    // Set push constant range
    SetPushConstants {
        stages: wgpu::ShaderStages,
        size: usize,
        global_offset: usize,
        local_offset: usize,
    },

    // Draw command without index buffer
    Draw {
        vertices: Range<u32>,
        instances: Range<u32>,
    },

    // Draw command with the current bound index buffer
    DrawIndexed {
        indices: Range<u32>,
        instances: Range<u32>,
    },

    // Indirect draw command without index buffer
    DrawIndirect {
        buffer: &'a DrawIndirectBuffer,
        element: usize,
    },

    // Indirect draw command with the current bound index buffer
    DrawIndexedIndirect {
        buffer: &'a DrawIndexedIndirectBuffer,
        element: usize,
    },
}

// Keep track of all compute commands that we call upon the compute pass
pub(crate) enum ComputeCommand<'a> {
    // Bind compute shader
    BindShader(&'a ComputeShader),

    // Set bind group
    SetBindGroup(u32, Arc<BindGroup>),

    // Set push constant range
    SetPushConstants {
        size: usize,
        global_offset: usize,
        local_offset: usize,
    },

    // Dispatch the compute pipeline
    Dispatch {
        x: u32,
        y: u32,
        z: u32,
    },
}

// Record the render commands to the given render pass
pub(crate) fn record_render_commands<
    'r,
    C: ColorLayout,
    DS: DepthStencilLayout,
>(
    mut render_pass: wgpu::RenderPass<'r>,
    push_constants: Vec<u8>,
    render_commands: &'r [RenderCommand<'r, C, DS>],
) {
    for render_command in render_commands {
        match render_command {
            RenderCommand::BindPipeline(pipeline) => {
                render_pass.set_pipeline(pipeline.pipeline());
            }

            RenderCommand::SetVertexBuffer {
                slot,
                buffer,
                start,
                end,
            } => {
                let bound = (*start, *end);
                render_pass.set_vertex_buffer(
                    *slot,
                    buffer.raw().slice(bound),
                )
            }

            RenderCommand::SetIndexBuffer { buffer, start, end } => {
                let bound = (*start, *end);
                render_pass.set_index_buffer(
                    buffer.raw().slice(bound),
                    wgpu::IndexFormat::Uint32,
                );
            }

            RenderCommand::SetBindGroup(index, bind_group) => {
                render_pass.set_bind_group(*index, &bind_group, &[]);
            }

            RenderCommand::SetPushConstants {
                stages,
                size,
                global_offset,
                local_offset,
            } => {
                let start = *global_offset;
                let end = global_offset + size;
                let data = &push_constants[start..end];
                render_pass.set_push_constants(
                    *stages,
                    *local_offset as u32,
                    data,
                );
            }

            RenderCommand::Draw {
                vertices,
                instances,
            } => {
                render_pass.draw(vertices.clone(), instances.clone());
            }

            RenderCommand::DrawIndexed { indices, instances } => {
                render_pass.draw_indexed(
                    indices.clone(),
                    0,
                    instances.clone(),
                );
            }

            RenderCommand::DrawIndirect { buffer, element } => {
                let indirect_offset = element * buffer.stride();
                render_pass.draw_indirect(
                    buffer.raw(),
                    indirect_offset as u64,
                )
            }

            RenderCommand::DrawIndexedIndirect {
                buffer,
                element,
            } => {
                let indirect_offset = element * buffer.stride();
                render_pass.draw_indexed_indirect(
                    buffer.raw(),
                    indirect_offset as u64,
                )
            }
        }
    }
}

// Record the compute commands to the given compute pass
pub(crate) fn record_compute_commands<'r>(
    mut compute_pass: wgpu::ComputePass<'r>,
    push_constants: Vec<u8>,
    compute_commands: &'r [ComputeCommand<'r>],
) {
    for compute_command in compute_commands {
        match compute_command {
            ComputeCommand::BindShader(shader) => {
                compute_pass.set_pipeline(shader.pipeline());
            }

            ComputeCommand::SetBindGroup(index, bind_group) => {
                compute_pass.set_bind_group(*index, &bind_group, &[]);
            }

            ComputeCommand::SetPushConstants {
                size,
                global_offset,
                local_offset,
            } => {
                let start = *global_offset;
                let end = global_offset + size;
                let data = &push_constants[start..end];
                compute_pass
                    .set_push_constants(*local_offset as u32, data);
            }

            ComputeCommand::Dispatch { x, y, z } => {
                compute_pass.dispatch_workgroups(*x, *y, *z);
            }
        }
    }
}
