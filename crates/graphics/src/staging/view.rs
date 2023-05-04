use crate::{Graphics, Texture, Buffer, GpuPod};
use std::{marker::PhantomData, sync::atomic::Ordering};
use utils::AtomicBitSet;

pub struct StagingView<'a> {
    pub(super) index: usize,
    pub(super) used: &'a AtomicBitSet,
    pub(super) staging: &'a wgpu::Buffer,
    pub(super) view: Option<wgpu::BufferView<'a>>,
}

impl<'a> AsRef<[u8]> for StagingView<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.view.as_ref().unwrap()
    }
}

impl<'a> Drop for StagingView<'a> {
    fn drop(&mut self) {
        self.view.take().unwrap();
        self.used.remove(self.index, Ordering::Relaxed);
        self.staging.unmap();
    }
}

// Used for mappable buffer writes ONLY
pub struct StagingViewWrite<'a> {
    pub(crate) write: wgpu::QueueWriteBufferView<'a>,
}

impl<'a> AsMut<[u8]> for StagingViewWrite<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.write
    }
}

// Used for mappable texture reads
pub struct TextureStagingView<'a> {
    pub(super) index: usize,
    pub(super) used: &'a AtomicBitSet,
    pub(super) staging: &'a wgpu::Buffer,
    pub(super) view: Option<wgpu::BufferView<'a>>,
}

impl<'a> AsRef<[u8]> for TextureStagingView<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.view.as_ref().unwrap()
    }
}

impl<'a> Drop for TextureStagingView<'a> {
    fn drop(&mut self) {
        self.view.take().unwrap();
        self.used.remove(self.index, Ordering::Relaxed);
        self.staging.unmap();
    }
}


// Used for mappable texture writes ONLY
pub struct TextureStagingViewWrite<'a> {
    pub(super) index: usize,
    pub(super) graphics: &'a Graphics,
    pub(super) encoder: Option<wgpu::CommandEncoder>,
    pub(super) used: &'a AtomicBitSet,
    pub(super) staging: &'a wgpu::Buffer,
    pub(crate) image_copy_buffer: Option<wgpu::ImageCopyBuffer<'a>>,
    pub(crate) image_copy_texture: Option<wgpu::ImageCopyTexture<'a>>,
    pub(crate) copy_size: wgpu::Extent3d,
    pub(super) view: Option<wgpu::BufferViewMut<'a>>,
}

impl<'a> AsMut<[u8]> for TextureStagingViewWrite<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.view.as_mut().unwrap().as_mut()
    }
}

impl<'a> Drop for TextureStagingViewWrite<'a> {
    fn drop(&mut self) {
        self.view.take().unwrap();
        self.used.remove(self.index, Ordering::Relaxed);
        self.staging.unmap();

        let mut encoder = self.encoder.take().unwrap();

        encoder.copy_buffer_to_texture(
            self.image_copy_buffer.take().unwrap(),
            self.image_copy_texture.take().unwrap(), 
            self.copy_size
        );

        self.graphics.reuse([encoder]);
    }
}
