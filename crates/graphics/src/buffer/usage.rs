
// How we shall access the buffer
// These buffer usages do not count the initial buffer creation phase
// Anything related to the device access is a hint since you can always access stuff
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BufferUsage {
    // Specifies what the device can do with the buffer
    pub hint_device_write: bool,
    pub hint_device_read: bool,

    // Specifies what the host can do do with the buffer
    pub host_write: bool,
    pub host_read: bool,
}

impl BufferUsage {    
    // Device local buffer usage. Not host visible
    pub fn device_local_usage() -> Self {
        Self {
            hint_device_write: true,
            hint_device_read: true,
            host_write: false,
            host_read: false,
        }
    }
    
    // Common buffer usage. Allows you to do anything
    pub fn common_device_usage() -> Self {
        Self {
            hint_device_write: true,
            hint_device_read: true,
            host_write: true,
            host_read: true,
        }
    }
    
    // Buffer usage to upload data to the GPU
    pub fn upload_to_device_usage() -> Self {
        Self {
            hint_device_write: false,
            hint_device_read: true,
            host_write: true,
            host_read: false,
        }
    }
    
    // Buffer usage to download data from the GPU
    pub fn download_from_device_usage() -> Self {
        Self {
            hint_device_write: true,
            hint_device_read: false,
            host_write: false,
            host_read: true,
        }
    }
}

impl Default for BufferUsage {
    fn default() -> Self {
        Self::common_device_usage()
    }
}
