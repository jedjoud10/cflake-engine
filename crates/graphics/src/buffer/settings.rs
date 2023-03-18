// Some settings that tell us how exactly we should create the buffer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Dynamic buffers are buffers that can be read and modified
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital,

    // Resizable buffers can be re-allocated to whatever capacity needed
    #[default]
    Resizable,
}

bitflags::bitflags! {
    // How exactly are we going to use the buffer?
    pub struct BufferUsage: u8 {
        // This buffer will be used as a storage buffer that we can read/write to
        const STORAGE = 1;

        // Data can be copied from the buffer on the GPU side
        const COPY_SRC = 2;

        // Data can be copied into the buffer on the GPU side
        const COPY_DST = 4;

        // The buffer can be used for reading GPU data back
        // Example: Data generated from a compute shader read back to the CPU
        const READ = 8 | Self::COPY_SRC.bits;

        // The buffer can be used to send data to the GPU
        // Example: Non-readable vertex buffers
        const WRITE = 16 | Self::COPY_DST.bits;
    }
}

impl Default for BufferUsage {
    fn default() -> Self {
        Self::READ | Self::COPY_DST
    }
}
