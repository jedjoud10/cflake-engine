use crate::{
    BufferInfo, ColorLayout, DepthStencilLayout, GraphicsPipeline,
    TriangleBuffer, UniformBuffer,
};
use std::{
    ops::{Bound, Range},
    sync::Arc,
};
use wgpu::BindGroup;

// Keep track of all render commands that we call upon the render pass
// The whole reason I have to delegate this to a command type system is because
// the render pass requires the BindGroups to live longer than itself, and I couldn't make it work
pub(crate) enum RenderCommand<
    'a,
    C: ColorLayout,
    DS: DepthStencilLayout,
> {
    // Bind graphics pipeline
    BindPipeline(&'a GraphicsPipeline<C, DS>),

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
        local_offset: u32,
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
}

// Record the render commands to the given render pass
pub(crate) fn record<'r, C: ColorLayout, DS: DepthStencilLayout>(
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
                    *local_offset,
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
        }
    }
}
