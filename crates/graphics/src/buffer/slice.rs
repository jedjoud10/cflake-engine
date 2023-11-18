use crate::{Buffer, GpuPod};

// An immutable buffer slice is a region of an immutable buffer
pub struct BufferSlice<'a> {
    buffer: &'a wgpu::Buffer,
    offset: usize,
    length: usize,
}

// A mutable buffer slice is a region of a mutable buffer
pub struct BufferSliceMut<'a> {
    buffer: &'a wgpu::Buffer,
    offset: usize,
    length: usize,
}

pub struct HangYourself;

impl HangYourself {
    pub fn testino(&self) {}
}

impl<T: GpuPod, const TYPE: u32> AsRef<HangYourself> for Buffer<T, TYPE> {
    fn as_ref(&self) -> &HangYourself {
        todo!()
    }
}

/*
impl<T: GpuPod, const TYPE: u32> std::ops::Deref for Buffer<T, TYPE> {
    type Target = HangYourself;

    fn deref(&self) -> &Self::Target {
        HangYourself
    }
}
*/
