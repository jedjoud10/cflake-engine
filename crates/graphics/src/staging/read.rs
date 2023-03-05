use std::marker::PhantomData;
use wgpu::{CommandEncoder, BufferAddress, BufferView};
use crate::{Graphics};

// This is the view returned from the download() method of the staging pool
// This allows us to read the data of the given buffer at the given offset and slice
// T is target, it could either be a buffer or a texture
pub(crate) struct StagingView<'a> {
    // API, encoder, and target
    graphics: &'a Graphics,
    encoder: &'a CommandEncoder,

    // Memory parameters
    dst_offset: BufferAddress,
    staging_offset: BufferAddress,
    size: BufferAddress,

    // WGPU buffer view into the staging buffer
    staging: &'a wgpu::Buffer,
    view: BufferView<'a>,
}

impl<'a> AsRef<[u8]> for StagingView<'a> {
    fn as_ref(&self) -> &[u8] {
        todo!()
    }
}

impl<'a> Drop for StagingView<'a> {
    fn drop(&mut self) {
        todo!()
    }
}