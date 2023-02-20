use crate::{
    ColorLayout, DepthStencilLayout, GraphicsPipeline,
    TriangleBuffer, UntypedBuffer,
};
use std::{ops::Range, sync::Arc};
use wgpu::BindGroup;

// Keep track of all render commands that we call upon the render pass
// The whole reason I have to delegate this to a command type system is because
// the render pass requires the BindGroups to live longer than itself, and I couldn't make it work
pub enum RenderCommand<'a, C: ColorLayout, DS: DepthStencilLayout> {
    BindPipeline(&'a GraphicsPipeline<C, DS>),
    SetVertexBuffer(u32, UntypedBuffer<'a>),
    SetIndexBuffer(&'a TriangleBuffer<u32>),
    SetBindGroup(u32, Arc<BindGroup>),
    Draw {
        vertices: Range<u32>,
        instances: Range<u32>,
    },
    DrawIndexed {
        indices: Range<u32>,
        instances: Range<u32>,
    },
}

// Record the render commands to the given render pass
pub fn record<'r, C: ColorLayout, DS: DepthStencilLayout>(
    mut render_pass: wgpu::RenderPass<'r>,
    render_commands: &'r [RenderCommand<'r, C, DS>],
) {
    for render_command in render_commands {
        match render_command {
            RenderCommand::BindPipeline(pipeline) => {
                render_pass.set_pipeline(pipeline.pipeline());
            }

            RenderCommand::SetVertexBuffer(slot, buffer) => {
                render_pass
                    .set_vertex_buffer(*slot, buffer.raw().slice(..))
            }

            RenderCommand::SetIndexBuffer(buffer) => {
                render_pass.set_index_buffer(
                    buffer.raw().slice(..),
                    wgpu::IndexFormat::Uint32,
                );
            }

            RenderCommand::SetBindGroup(index, bind_group) => {
                render_pass.set_bind_group(*index, &bind_group, &[]);
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
