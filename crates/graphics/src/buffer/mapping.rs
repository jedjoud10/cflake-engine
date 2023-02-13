use crate::{GpuPodRelaxed, Buffer, StagingPool};

// Allows  us to read the buffer as if it were an immutably slice
pub struct BufferView<'a, T: GpuPodRelaxed, const TYPE: u32> {
    pub(crate) buffer: &'a Buffer<T, TYPE>,
    pub(crate) data: wgpu::BufferView<'a>,
    pub(crate) staging: &'a StagingPool,
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> BufferView<'a, T, TYPE> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        todo!()
        //&*self.data
    }
}

// Allows us to read the buffer as if it were a mutable slice
pub enum BufferViewMut<'a, T: GpuPodRelaxed, const TYPE: u32> {
    // The buffer's staging buffer is mapped mutably
    Mapped {
        buffer: &'a mut Buffer<T, TYPE>,
    }, 

    // Read the buffer's data to the CPU for reading/writing
    Cloned {
        buffer: &'a mut Buffer<T, TYPE>,
        data: Vec<T>
    },
}

impl<'a, T: GpuPodRelaxed, const TYPE: u32> BufferViewMut<'a, T, TYPE> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        todo!()
    }

    // Get a mutable slice that we can read/write from
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        todo!()
    }
}