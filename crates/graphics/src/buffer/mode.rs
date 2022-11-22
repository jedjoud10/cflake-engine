// Some settings that tell us how exactly we should create the buffer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Static buffers are only created once, and they can never be modified ever again
    Static {
        persistent: bool,
        map_read: bool,
    },

    // Dynamic buffers are like static buffers, but they allow the user to mutate each element
    Dynamic {
        persistent: bool,
        map_write: bool,
        map_read: bool,
    },

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital {
        persistent: bool,
        map_write: bool,
        map_read: bool,
    },

    // Resizable buffers can be resized to whatever length needed
    Resizable {
        persistent: bool,
        map_write: bool,
        map_read: bool,
    },
}

impl Default for BufferMode {
    fn default() -> Self {
        Self::Resizable {
            persistent: false,
            map_write: false,
            map_read: false
        }
    }
}


impl BufferMode {
    // Can we read from an arbitrary buffer that uses this buffer mode?
    pub fn read_permission(&self) -> bool {
        true
    }

    // Can we write to an arbitrary buffer that uses this buffer mode?
    pub fn write_permission(&self) -> bool {
        match self {
            BufferMode::Static { .. } => false,
            _ => true,
        }
    }

    // Can we modify the LENGTH of an arbitrary buffer that uses this buffer mode?
    pub fn modify_length_permission(&self) -> bool {
        match self {
            BufferMode::Resizable { .. } | BufferMode::Parital { .. } => true,
            _ => false,
        }
    }

    // Can we reallocate an arbitrary buffer that uses this buffer mode?
    pub fn reallocate_permission(&self) -> bool {
        match self {
            BufferMode::Resizable { .. } => true,
            _ => false,
        }
    }

    // Check if we can map the buffer persistently
    pub fn map_persistent_permission(&self) -> bool {
        match self {
            BufferMode::Static { persistent, .. }
            | BufferMode::Dynamic { persistent, .. }
            | BufferMode::Parital { persistent, .. }
            | BufferMode::Resizable { persistent, .. } => *persistent,
        }
    }

    // Check if we can map the buffer for reading
    pub fn map_read_permission(&self) -> bool {
        match self {
            BufferMode::Static { map_read, .. }
            | BufferMode::Dynamic { map_read, .. }
            | BufferMode::Parital { map_read, .. }
            | BufferMode::Resizable { map_read, .. } => *map_read,
        }
    }

    // Check if we can map the buffer for writing
    pub fn map_write_permission(&self) -> bool {
        match self {
            BufferMode::Dynamic { map_write, .. } 
            | BufferMode::Parital { map_write, .. }
            | BufferMode::Resizable { map_write, .. } => {
                *map_write
            }
            BufferMode::Static { .. } => false,
        }
    }
}