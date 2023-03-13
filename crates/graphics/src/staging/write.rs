use crate::Graphics;
use std::marker::PhantomData;
use wgpu::{Buffer, BufferAddress, BufferViewMut, CommandEncoder, QueueWriteBufferView};

// This is the view returned from the map_buffer_write() method of the staging pool
pub struct StagingViewWrite<'a> {
    pub(crate) write: QueueWriteBufferView<'a>,
}

impl<'a> AsRef<[u8]> for StagingViewWrite<'a> {
    fn as_ref(&self) -> &[u8] {
        log::error!("Trying to read from StagingViewWrite buffer. Contents are undefined/zeroed");
        &self.write
    }
}

impl<'a> AsMut<[u8]> for StagingViewWrite<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.write
    }
}