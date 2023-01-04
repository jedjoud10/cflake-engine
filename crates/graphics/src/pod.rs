pub unsafe trait GpuPodRelaxed:
    bytemuck::AnyBitPattern
    + bytemuck::NoUninit
    + Clone
    + Copy
    + Sync
    + Send
    + 'static
{
}
unsafe impl<
        T: Clone
            + Copy
            + Sync
            + Send
            + bytemuck::AnyBitPattern
            + bytemuck::NoUninit
            + 'static,
    > GpuPodRelaxed for T
{
}

pub unsafe trait GpuPod:
    bytemuck::Pod
    + bytemuck::Zeroable
    + Clone
    + Copy
    + Sync
    + Send
    + 'static
{
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
