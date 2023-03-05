use crate::Graphics;
use std::marker::PhantomData;
use wgpu::{Buffer, BufferAddress, BufferViewMut, CommandEncoder};

// This is the view returned from the upload() method of the staging pool
// This allows us to write to the given buffer (it will submit this write when this gets dropped)
// We cannot read from this staging view. This is only used for writing into the buffer
pub struct StagingViewWrite<'a> {
    // API, encoder, and target
    graphics: &'a Graphics,

    // Memory parameters
    dst_offset: BufferAddress,
    staging_offset: BufferAddress,
    size: BufferAddress,

    // WGPU mutable buffer view into the staging buffer
    staging: &'a Buffer,
    view: BufferViewMut<'a>,
}

impl<'a> AsRef<[u8]> for StagingViewWrite<'a> {
    fn as_ref(&self) -> &[u8] {
        log::error!("Trying to read from MappedStatingWrite buffer. Contents are undefined/zeroed");
        &self.view
    }
}

impl<'a> AsMut<[u8]> for StagingViewWrite<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.view
    }
}

impl<'a> Drop for StagingViewWrite<'a> {
    fn drop(&mut self) {
        todo!()
    }
}
