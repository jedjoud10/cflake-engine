use std::marker::PhantomData;
use wgpu::{CommandEncoder, BufferAddress, BufferView};
use crate::{Graphics, Target};

// This is the view returned from the download() method of the staging pool
// This allows us to read the data of the given buffer at the given offset and slice
// T is target, it could either be a buffer or a texture
pub struct StagingView<'a, T: Target> {
    // API, encoder, and target
    graphics: &'a Graphics,
    encoder: &'a CommandEncoder,
    _phantom: PhantomData<&'a T>,

    // Memory parameters
    dst_offset: BufferAddress,
    staging_offset: BufferAddress,
    size: BufferAddress,

    // WGPU buffer view into the staging buffer
    staging: &'a wgpu::Buffer,
    view: BufferView<'a>,
}

impl<'a, T: Target> AsRef<[u8]> for StagingView<'a, T> {
    fn as_ref(&self) -> &[u8] {
        todo!()
    }
}

impl<'a, T: Target> Drop for StagingView<'a, T> {
    fn drop(&mut self) {
        todo!()
    }
}