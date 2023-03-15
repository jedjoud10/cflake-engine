use crate::{
    ModuleVisibility, PushConstantBytesError, PushConstantLayout,
    ReflectedShader,
};
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
    // Push a sub-region of push constant data to be stored afterwards
    pub fn push(
        &mut self,
        bytes: &[u8],
        mut offset: u32,
        visibility: ModuleVisibility,
    ) -> Result<(), PushConstantBytesError> {
        // Make sure we have bytes to write with
        if bytes.is_empty() {
            return Err(PushConstantBytesError::NoBytes);
        }

        // Make sure we won't overwrite the buffer
        if (bytes.len() + offset as usize) > self.data.len() {
            return Err(
                PushConstantBytesError::OffsetOrSizeIsTooLarge,
            );
        }

        // Make sure the visibility matches up
        match (visibility, self.layout) {
            (
                ModuleVisibility::Vertex,
                PushConstantLayout::SplitVertexFragment { .. },
            ) => {}
            (
                ModuleVisibility::Fragment,
                PushConstantLayout::SplitVertexFragment {
                    vertex,
                    ..
                },
            ) => offset += vertex.get(),
            (a, PushConstantLayout::Single(_, b)) if a == b => {}
            _ => return Err(PushConstantBytesError::NotAsDefined),
        }

        // Set the bytes properly
        let start = offset as usize;
        let end = start + bytes.len();
        self.data[start..end].copy_from_slice(bytes);
        Ok(())
    }
}
