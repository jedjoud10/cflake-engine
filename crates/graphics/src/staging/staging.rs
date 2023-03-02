use crate::{Graphics, StagingView, StagingViewWrite, Buffer, GpuPod, GpuPodRelaxed};
use parking_lot::Mutex;
use std::{num::NonZeroU64, ops::DerefMut, sync::Arc, marker::PhantomData};
use utils::ConcVec;
use wgpu::{CommandEncoder, Maintain};

// A target of staging download / upload events
// Could either be a texture or a buffer
pub trait Target {}

impl Target for wgpu::Buffer {
}

// Helper struct that will temporarily store mapped buffers so we can have
// StagingView / StagingViewMut that we can read and write from 
// This will re-use unmapped buffers to avoid many many buffer creations

pub struct StagingPool {
}

impl StagingPool {
    // Create a new staging belt for upload / download
    pub fn new() -> Self {
        Self {
        }
    }

    // Map a target for writing only (maps an intermediate staging buffer)
    // Src target must have the COPY_SRC buffer usage flag
    pub fn map_read<'a, T: Target>(
        &'a self,
        target: &T,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferAddress,
    ) -> Option<StagingView<'a, T>> {
        None
    }

    // Map a target for writing only (maps an intermediate staging buffer)
    // Src target must have the COPY_DST buffer usage flag
    pub fn map_write<'a, T: Target>(
        &'a self,
        target: &T,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferAddress,
    ) -> Option<StagingViewWrite<'a, T>> {
        None
    }

    // Request an immediate target write
    // Src target must have the COPY_DST buffer usage flag
    pub fn write<'a, T: Target>(
        &'a self,
        target: &T,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        src: &[u8],
    ) {
        ()
    }

    // Request an immediate target read
    // Src target must have the COPY_SRC buffer usage flag
    pub fn read<'a, T: Target>(
        &'a self,
        target: &T,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        dst: &mut [u8],
    ) {
        ()
    }
}
