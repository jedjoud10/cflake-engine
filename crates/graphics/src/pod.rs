// Plain old data type internally used by Vulkan objects
// This does NOT implement bytemuck::Pod since it might contain padding bytes (structs)
pub trait GpuPod:
    bytemuck::AnyBitPattern
    + bytemuck::NoUninit
    + Clone
    + Copy
    + Sync
    + Send
    + 'static
{
}
impl<
        T: Clone
            + Copy
            + Sync
            + Send
            + bytemuck::AnyBitPattern
            + bytemuck::NoUninit
            + 'static,
    > GpuPod for T
{
}
