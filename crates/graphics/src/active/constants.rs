use crate::{
    Graphics, ModuleVisibility, PushConstantBytesError,
    PushConstantLayout, ReflectedShader, SetPushConstantsError,
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

// Create some push constants that the user can set
pub(super) fn handle_push_constants<'b>(
    reflected: Arc<ReflectedShader>,
    push_constant: &'b mut Vec<u8>,
    push_constant_global_offset: &mut usize,
    callback: impl FnOnce(&mut PushConstants<'b>),
) -> Option<PushConstantLayout> {
    // Don't set the push constants if we don't have any to set
    let Some(layout) = reflected.push_constant_layout else {
        return None;
    };

    // Make sure we have enough bytes to store the push constants
    let pc = push_constant.len() - *push_constant_global_offset;
    if pc < 1024 {
        push_constant.extend(std::iter::repeat(0).take(1024));
    }

    // Get the max size that we must allocate (at minimum) to be able to use ALL the defined push constants
    let size = layout.size().get();

    // Get the data that we will use
    let start = *push_constant_global_offset as usize;
    let end = size as usize + start;
    let data = &mut push_constant[start..end];

    // Create push constants that we can set
    let mut push_constants = PushConstants { data, layout };

    // Let the user modify the push constant
    callback(&mut push_constants);
    *push_constant_global_offset += size as usize;
    return Some(layout);
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
            _ => {
                return Err(
                    PushConstantBytesError::VisibilityNotValid(
                        visibility,
                        self.layout.visibility(),
                    ),
                )
            }
        }

        // Set the bytes properly
        let start = offset as usize;
        let end = start + bytes.len();
        self.data[start..end].copy_from_slice(bytes);
        Ok(())
    }
}
