use arrayvec::ArrayVec;
use itertools::Itertools;

use crate::{
    ReflectedShader, ModuleVisibility,
};
use std::{marker::PhantomData, sync::Arc, ops::RangeBounds};

// Push constants are tiny bits of memory that are going to get stored directly in a command encoder
// They are mostly used to upload bits of data very rapidly to use within shaders
pub struct PushConstants<'a> {
    pub(crate) reflected: Arc<ReflectedShader>,
    pub(crate) offsets: Vec<u32>,
    pub(crate) sizes: Vec<usize>,
    pub(crate) data: &'a mut [u8],
    pub(crate) stages: Vec<wgpu::ShaderStages>,
}

impl PushConstants<'_> {
    // Set the given push constants of a given range and push them
    pub fn push(
        &mut self,
        bytes: &[u8],
        offset: u32,
        visibility: ModuleVisibility,
    ) {
    }
}