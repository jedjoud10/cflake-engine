use std::marker::PhantomData;

use parking_lot::MappedMutexGuard;

use crate::{GpuPodRelaxed, Buffer, StagingPool};

// Allows  us to read the buffer as if it were an immutably slice
pub struct BufferView<'a, T: GpuPodRelaxed, const TYPE: u32> {
    pub(crate) _phantom: PhantomData<&'a Buffer<T, TYPE>>,
    pub(crate) data: wgpu::BufferView<'a>,
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> BufferView<'a, T, TYPE> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        let bytes = &*self.data;
        bytemuck::cast_slice(bytes)
    }
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> AsRef<[T]> for BufferView<'a, T, TYPE> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

// Allows us to read the buffer as if it were a mutable slice
pub enum BufferViewMut<'a, T: GpuPodRelaxed, const TYPE: u32> {
    // The buffer's staging buffer is mapped mutably
    // Only used when WRITING ONLY
    Mapped {
        buffer: &'a mut Buffer<T, TYPE>,
        data: wgpu::QueueWriteBufferView<'a>,
    }, 

    // Read the buffer's data to the CPU for reading/writing
    // Used when the buffer is readable AND writable 
    Cloned {
        buffer: &'a mut Buffer<T, TYPE>,
        data: Vec<T>,
    },
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> BufferViewMut<'a, T, TYPE> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        match self {
            BufferViewMut::Mapped { data, .. } => {
                let bytes = &*data;
                bytemuck::cast_slice(bytes)
            },
            BufferViewMut::Cloned { data, .. } => &data,
        }
    }

    // Get a mutable slice that we can read/write from
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        match self {
            BufferViewMut::Mapped { data, .. } => {
                let bytes = &mut **data;
                bytemuck::cast_slice_mut(bytes)
            },
            BufferViewMut::Cloned { data, .. } => data,
        }
    }
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> AsRef<[T]> for BufferViewMut<'a, T, TYPE> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> AsMut<[T]> for BufferViewMut<'a, T, TYPE> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> Drop for BufferViewMut<'a, T, TYPE> {
    fn drop(&mut self) {
        todo!()
    }
}