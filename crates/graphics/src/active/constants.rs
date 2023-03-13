use crate::{ModuleVisibility, ReflectedShader};
use arrayvec::ArrayVec;
use itertools::Itertools;
use std::{marker::PhantomData, ops::RangeBounds, sync::Arc};

pub(crate) struct PushConstantRange {
    pub(crate) offset: u32,
    pub(crate) size: usize,
    pub(crate) stages: wgpu::ShaderStages,
}

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

        self.ranges.push(PushConstantRange {
            offset,
            size: bytes.len(),
            stages: match visibility {
                ModuleVisibility::Vertex => {
                    wgpu::ShaderStages::VERTEX
                }
                ModuleVisibility::Fragment => todo!(),
                ModuleVisibility::VertexFragment => todo!(),
                ModuleVisibility::Compute => todo!(),
            },
        });
    }
}
