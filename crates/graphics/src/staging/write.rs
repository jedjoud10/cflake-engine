use std::marker::PhantomData;
use wgpu::{CommandEncoder, BufferAddress, Buffer, BufferViewMut};
use crate::{Graphics, StagingTarget};

// This is the view returned from the upload() method of the staging pool
// This allows us to write to the given buffer (it will submit this write when this gets dropped)
pub struct StagingViewWrite<'a> {
    // API, encoder, and target
    graphics: &'a Graphics,
    src: StagingTarget<'a>,

    // Memory parameters
    dst_offset: BufferAddress,
    staging_offset: BufferAddress,
    size: BufferAddress,

    // WGPU mutable buffer view into the staging buffer 
    staging: &'a Buffer,
    view: BufferViewMut<'a>,
}

impl<'a> AsMut<[u8]> for StagingViewWrite<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        todo!()
    }
}

impl<'a> AsRef<[u8]> for StagingViewWrite<'a> {
    fn as_ref(&self) -> &[u8] {
        todo!()
    }
}

impl<'a> Drop for StagingViewWrite<'a> {
    fn drop(&mut self) {
        todo!()
    }
}