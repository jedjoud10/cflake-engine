use crate::Graphics;
use std::{marker::PhantomData, sync::atomic::Ordering};
use utils::AtomicBitSet;
use wgpu::{BufferAddress, BufferView, CommandEncoder};

// This is the view returned from the map_buffer_read() method of the staging pool
// This allows us to read the data of the given buffer at the given offset and slice
pub struct StagingView<'a> {
    // WGPU buffer view into the staging buffer
    pub(super) index: usize,
    pub(super) states: &'a AtomicBitSet,
    pub(super) staging: &'a wgpu::Buffer,
    pub(super) view: Option<BufferView<'a>>,
}

impl<'a> AsRef<[u8]> for StagingView<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.view.as_ref().unwrap()
    }
}

impl<'a> Drop for StagingView<'a> {
    fn drop(&mut self) {
        self.view.take().unwrap();
        self.states.remove(self.index, Ordering::Relaxed);
        self.staging.unmap();
    }
}
