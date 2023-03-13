use crate::Graphics;
use std::{marker::PhantomData, sync::atomic::Ordering};
use utils::AtomicBitSet;
use wgpu::{BufferAddress, BufferView, CommandEncoder};

// This is the view returned from the download() method of the staging pool
// This allows us to read the data of the given buffer at the given offset and slice
// T is target, it could either be a buffer or a texture
pub(crate) struct StagingView<'a> {
    // API, encoder, and target
    pub(crate) graphics: &'a Graphics,

    // Memory parameters
    pub(crate) dst_offset: BufferAddress,
    pub(crate) staging_offset: BufferAddress,
    pub(crate) size: BufferAddress,

    // WGPU buffer view into the staging buffer
    pub(super) index: usize,
    pub(super) states: &'a AtomicBitSet,
    pub(super) staging: &'a wgpu::Buffer,
    pub(super) view: BufferView<'a>,
}

impl<'a> AsRef<[u8]> for StagingView<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.view
    }
}

impl<'a> Drop for StagingView<'a> {
    fn drop(&mut self) {
        self.states.remove(self.index, Ordering::Relaxed);
        self.staging.unmap();
    }
}
