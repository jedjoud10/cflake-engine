// Buffer settings that show usage and mode
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct BufferSettings {
    pub mapping: BufferMapping,
    pub mode: BufferMode,
}

// How we will map the buffer when reading / writing to it
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct BufferMapping {
    pub persistent: bool,
    pub map_write: bool,
    pub map_read: bool,
}

// Some settings that tell us how exactly we should create the buffer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Dynamic buffers are only created once, but they allow the user to mutate each element
    #[default]
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital,

    // Resizable buffers can be resized to whatever length needed
    Resizable,
}