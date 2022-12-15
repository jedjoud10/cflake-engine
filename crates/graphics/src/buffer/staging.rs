// A staging pool is used to transfer data between GPU and CPU memory
// This is a ring buffer of host visible device local buffers
// Stolen from https://docs.rs/vulkano/latest/vulkano/buffer/cpu_pool/struct.CpuBufferPool.html
pub(crate) struct StagingPool {

}