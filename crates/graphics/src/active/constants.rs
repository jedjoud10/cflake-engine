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
    // There's no validation here since we do it using bitsets later on
    pub fn push(
        &mut self,
        bytes: &[u8],
        offset: u32,
        visibility: ModuleVisibility,
    ){
        self.data.copy_from_slice(bytes);
        self.ranges.push(PushConstantRange {
            visibility,
            start: offset,
            end: offset + bytes.len() as u32,
        });
    }
}
