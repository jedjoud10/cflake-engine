use crate::{
    Buffer, GpuPodRelaxed, StagingPool, StagingView, StagingViewWrite,
};
use parking_lot::MappedMutexGuard;
use wgpu::CommandEncoder;
use std::marker::PhantomData;

// Allows  us to read the buffer as if it were an immutably slice
pub struct BufferView<'a, T: GpuPodRelaxed, const TYPE: u32> {
    pub(crate) buffer: &'a Buffer<T, TYPE>,
    pub(crate) data: StagingView<'a>,
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> BufferView<'a, T, TYPE> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        let bytes = self.data.as_ref();
        bytemuck::cast_slice(bytes)
    }
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> AsRef<[T]>
    for BufferView<'a, T, TYPE>
{
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> std::ops::Deref
    for BufferView<'a, T, TYPE>
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

// Allows us to read the buffer as if it were a mutable slice
pub enum BufferViewMut<'a, T: GpuPodRelaxed, const TYPE: u32> {
    // The buffer's staging buffer is mapped mutably
    // Only used when WRITING ONLY
    Mapped {
        buffer: PhantomData<&'a Buffer<T, TYPE>>,
        data: StagingViewWrite<'a>,
    },

    // Copy the buffer's data to the CPU for reading/writing
    // StagingViewWrite buffers cannot be read since MapMode is either Read or Write
    // So we need to have this to be able to do that whenever the buffer has the "read" usage on it
    Cloned {
        buffer: &'a mut Buffer<T, TYPE>,
        data: Vec<T>,
    },
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32>
    BufferViewMut<'a, T, TYPE>
{
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        match self {
            BufferViewMut::Mapped { data, .. } => {
                let bytes = data.as_ref();
                bytemuck::cast_slice(bytes)
            }
            BufferViewMut::Cloned { data, .. } => &data,
        }
    }

    // Get a mutable slice that we can read/write from
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        match self {
            BufferViewMut::Mapped { data, .. } => {
                let bytes = data.as_mut();
                bytemuck::cast_slice_mut(bytes)
            }
            BufferViewMut::Cloned { data, .. } => data,
        }
    }
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> AsRef<[T]>
    for BufferViewMut<'a, T, TYPE>
{
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> AsMut<[T]>
    for BufferViewMut<'a, T, TYPE>
{
    fn as_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> Drop
    for BufferViewMut<'a, T, TYPE>
{
    fn drop(&mut self) {
        match self {
            // Write the cloned data back into the buffer when we drop the view
            BufferViewMut::Cloned { buffer, data } => {
                buffer.write(&data, 0);
            }
            _ => {}
        }
    }
}
