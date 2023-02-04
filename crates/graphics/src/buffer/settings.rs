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

// How exactly are we going to use the buffer?
bitflags::bitflags! {
    pub struct BufferUsage: u32 {
        // The buffer is only going to be used for reading GPU data back
        // Example: Data generated from a compute shader read back to the CPU
        const READ = 0b01;

        // The buffer is only going to be used to send data to the GPU
        // Example: Non-readable vertex buffers
        const WRITE = 0b10;
    }
}
