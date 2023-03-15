// Plain old data type that can be sent to the gpu
// This is a bit of a hack tbh since bool doesn't implement
pub unsafe trait GpuPod:
    bytemuck::Pod + bytemuck::Zeroable + Sync + Send + 'static
{
    // Convert the data type to raw bytes
    fn into_bytes(&self) -> &[u8] {
        let ptr = self as *const Self;

        // This is safe since the type implements bytemuck::Pod, and we are only casting one element
        unsafe {
            core::slice::from_raw_parts(
                ptr as *const u8,
                Self::size(),
            )
        }
    }

    // Try converting raw bytes into self
    fn from_bytes(bytes: &[u8]) -> Self {
        let raw: &[Self] = bytemuck::cast_slice(bytes);
        debug_assert_eq!(raw.len(), 1);
        raw[0]
    }

    // Convert a slice of GpuPods into bytes
    fn slice_into_bytes(slice: &[Self]) -> &[u8] {
        bytemuck::cast_slice(slice)
    }

    // Convert a slice of bytes into GpuPods
    fn bytes_into_slice(bytes: &[u8]) -> &[Self] {
        bytemuck::cast_slice(bytes)
    }

    // Get the size of this POD
    fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    // Get the alignment value of this POD
    fn alignment() -> usize {
        std::mem::align_of::<Self>()
    }

    // Get the untyped GPU pod info
    fn info() -> GpuPodInfo {
        GpuPodInfo {
            size: Self::size(),
            alignment: Self::alignment(),
        }
    }
}

unsafe impl<
        T: Clone
            + Copy
            + Sync
            + Send
            + bytemuck::Pod
            + bytemuck::Zeroable
            + 'static,
    > GpuPod for T
{
}

// Gpu pod info simply contains the size and alignment of a GPU pod type
pub struct GpuPodInfo {
    size: usize,
    alignment: usize,
}

impl GpuPodInfo {
    // Get the size of this POD
    pub fn size(&self) -> usize {
        self.size
    }

    // Get the alignment value of this POD
    pub fn alignment(&self) -> usize {
        self.alignment
    }
}
