// Some settings that tell us how exactly we should create the buffer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Dynamic buffers are buffers that can be read and modified
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital,

    #[default]
    // Resizable buffers can be re-allocated to whatever capacity needed
    Resizable,
}
