use crate::{ModuleVisibility, ReflectedShader, PushConstantRange};
use arrayvec::ArrayVec;
use itertools::Itertools;
use std::{marker::PhantomData, ops::RangeBounds, sync::Arc};

// Push constants are tiny bits of memory that are going to get stored directly in a command encoder
// They are mostly used to upload bits of data very rapidly to use within shaders
pub struct PushConstants<'a> {
    pub(crate) reflected: Arc<ReflectedShader>,
    pub(crate) data: &'a mut [u8],
    pub(crate) ranges: Vec<PushConstantRange>,
}

impl PushConstants<'_> {
    // Set the given push constants of a given range and push them
    // TODO: Validate this shit
    pub fn push(
        &mut self,
        bytes: &[u8],
        offset: u32,
        visibility: ModuleVisibility,
    ) {
        self.data.copy_from_slice(bytes);

        let visibility = match visibility {
            ModuleVisibility::Vertex => wgpu::ShaderStages::VERTEX,
            ModuleVisibility::Fragment => wgpu::ShaderStages::FRAGMENT,
            ModuleVisibility::VertexFragment => wgpu::ShaderStages::VERTEX_FRAGMENT,
            ModuleVisibility::Compute => wgpu::ShaderStages::COMPUTE,
        };

        self.ranges.push(PushConstantRange {
            visibility: todo!(),
            start: todo!(),
            end: todo!(),
        });
    }
}
