use crate::{ModuleVisibility, PushConstantLayout, ReflectedShader};
use arrayvec::ArrayVec;
use itertools::Itertools;
use std::{marker::PhantomData, ops::RangeBounds, sync::Arc};

// Push constants are tiny bits of memory that are going to get stored directly in a command encoder
// They are mostly used to upload bits of data very rapidly to use within shaders
pub struct PushConstants<'a> {
    pub(crate) data: &'a mut [u8],
    pub(crate) layout: PushConstantLayout,
}

impl PushConstants<'_> {
    // Set the given push constants of a given range and push them
    pub fn push(
        &mut self,
        bytes: &[u8],
        mut offset: u32,
        visibility: ModuleVisibility,
    ) {
        if bytes.is_empty() {
            return;
        }

        if (bytes.len() + offset as usize) > self.data.len() {
            panic!()
        }

        match (visibility, self.layout) {
            (ModuleVisibility::Vertex, PushConstantLayout::VertexFragment { vertex, .. }) => {},
            (ModuleVisibility::Fragment, PushConstantLayout::VertexFragment { vertex, .. }) => offset += vertex,
            (ModuleVisibility::VertexFragment, PushConstantLayout::SharedVertexFragment(_)) => {},
            (ModuleVisibility::Compute, PushConstantLayout::Compute(_)) => {},
            _ => panic!()
        }

        let start = offset as usize;
        let end = start + bytes.len();
        self.data[start..end].copy_from_slice(bytes);
    }
}
