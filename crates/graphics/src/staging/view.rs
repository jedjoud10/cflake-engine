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

pub struct StagingViewWrite<'a> {
    pub(crate) write: wgpu::QueueWriteBufferView<'a>,
}

impl<'a> AsMut<[u8]> for StagingViewWrite<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.write
    }
}


pub struct TextureStagingView<'a> {
    pub(super) index: usize,
    pub(super) texture: &'a wgpu::Texture,
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

pub struct TextureStagingViewWrite<'a> {
    pub(super) used: &'a AtomicBitSet,
    pub(super) staging: &'a wgpu::Buffer,
    pub(super) view: Option<wgpu::BufferViewMut<'a>>,
}

impl<'a> AsMut<[u8]> for TextureStagingViewWrite<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        todo!()
    }
}
