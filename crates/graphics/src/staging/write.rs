use std::marker::PhantomData;
use wgpu::{CommandEncoder, BufferAddress, Buffer, BufferViewMut};
use crate::{Graphics, Target};

// This is the view returned from the upload() method of the staging pool
// This allows us to write to the given buffer (it will submit this write when this gets dropped)
pub struct StagingViewWrite<'a, T: Target> {
    // API, encoder, and target
    graphics: &'a Graphics,
    encoder: &'a CommandEncoder,
    _phantom: PhantomData<&'a mut T>,

    // Memory parameters
    dst_offset: BufferAddress,
    staging_offset: BufferAddress,
    size: BufferAddress,

    // WGPU mutable buffer view into the staging buffer 
    staging: &'a Buffer,
    view: BufferViewMut<'a>,
}

impl<'a, T: Target> AsMut<[u8]> for StagingViewWrite<'a, T> {
    fn as_mut(&mut self) -> &mut [u8] {
        todo!()
    }
}

impl<'a, T: Target> AsRef<[u8]> for StagingViewWrite<'a, T> {
    fn as_ref(&self) -> &[u8] {
        todo!()
    }
}

impl<'a, T: Target> Drop for StagingViewWrite<'a, T> {
    fn drop(&mut self) {
        todo!()
    }
}