use super::Recorder;
use ash::vk;

// Any type of command that can be applied
pub(crate) enum Command {
    Buffer(BufferCommand),
}

// A type of access
pub(crate) enum Access {
    Buffer(BufferAccess),
}

// Records the commands in the actual command buffer
pub(crate) trait InsertVkCommand {
    // Record the commands into the given command buffer
    unsafe fn insert(
        self,
        device: &ash::Device,
        buffer: vk::CommandBuffer,
    );
}

impl InsertVkCommand for Command {
    unsafe fn insert(
        self,
        device: &ash::Device,
        buffer: vk::CommandBuffer,
    ) {
        match self {
            Command::Buffer(x) => x.insert(device, buffer),
        }
    }
}

// Enum that contains all the types of commands that can be applied to buffers
pub(crate) enum BufferCommand {
    BindIndexBuffer {
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    },
    BindVertexBuffer {
        first_binding: u32,
        buffers: Vec<vk::Buffer>,
        offsets: Vec<vk::DeviceSize>,
    },
    CopyBuffer {
        src: vk::Buffer,
        dst: vk::Buffer,
        regions: Vec<vk::BufferCopy>,
    },
    CopyImageToBuffer {
        dst: vk::Buffer,
        src: vk::Image,
        layout: vk::ImageLayout,
        regions: Vec<vk::BufferImageCopy>,
    },
    FillBuffer {
        src: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
        data: u32,
    },
    UpdateBuffer {
        src: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
        data: Vec<u8>,
    },
}

// Enum that tells us how a buffer is accessed
#[derive(Debug)]
pub(crate) struct BufferAccess {
    pub(crate) buffer: vk::Buffer,
    pub(crate) mutable: bool,
    pub(crate) size: u64,
    pub(crate) offset: u64,
}

// Buffer commands
impl Recorder {
    // Add a new buffer command internally
    unsafe fn push_buffer_cmd(&mut self, cmd: BufferCommand) {
        self.state.commands.push(Command::Buffer(cmd));
    }

    // Add a new buffer access internally
    unsafe fn push_buffer_access(&mut self, access: BufferAccess) {
        self.state.access.push(Access::Buffer(access));
    }

    // Bind an index buffer to the command buffer render pass
    pub unsafe fn bind_index_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
        self.push_buffer_cmd(BufferCommand::BindIndexBuffer {
            buffer,
            offset,
            index_type,
        });
        self.push_buffer_access(BufferAccess {
            buffer,
            mutable: false,
            size: vk::WHOLE_SIZE,
            offset,
        });
    }

    // Bind vertex buffers to the command buffer render pass
    pub unsafe fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers: Vec<vk::Buffer>,
        offsets: Vec<vk::DeviceSize>,
    ) {
        for (i, &buffer) in buffers.iter().enumerate() {
            self.push_buffer_access(BufferAccess {
                buffer,
                mutable: false,
                size: vk::WHOLE_SIZE,
                offset: offsets[i],
            });
        }
        self.push_buffer_cmd(BufferCommand::BindVertexBuffer {
            first_binding,
            buffers,
            offsets,
        });
    }

    // Copy a buffer to another buffer in GPU memory
    pub unsafe fn copy_buffer(
        &mut self,
        src: vk::Buffer,
        dst: vk::Buffer,
        regions: Vec<vk::BufferCopy>,
    ) {
        for buffer_copy in regions.iter() {
            self.push_buffer_access(BufferAccess {
                buffer: src,
                mutable: false,
                size: buffer_copy.size,
                offset: buffer_copy.src_offset,
            });
            self.push_buffer_access(BufferAccess {
                buffer: dst,
                mutable: true,
                size: buffer_copy.size,
                offset: buffer_copy.dst_offset,
            });
        }
        self.push_buffer_cmd(BufferCommand::CopyBuffer {
            src,
            dst,
            regions,
        });
    }

    // Copy an image to a buffer in GPU memory
    pub unsafe fn copy_image_to_buffer(
        &mut self,
        buffer: vk::Buffer,
        image: vk::Image,
        layout: vk::ImageLayout,
        regions: Vec<vk::BufferImageCopy>,
    ) {
        for _buffer_image_copy in regions.iter() {
            //self.push_buffer_access(BufferAccess { buffer, mutable: true, size: buffer_image_copy., offset: buffer_image_copy.buffer_offset });
            //self.push_buffer_access(BufferAccess { buffer: dst, mutable: true, size: copy.size, offset: copy.dst_offset });
        }
        self.push_buffer_cmd(BufferCommand::CopyImageToBuffer {
            dst: buffer,
            src: image,
            layout,
            regions,
        });
    }

    // Clear a buffer to zero
    pub unsafe fn cmd_clear_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) {
        self.push_buffer_cmd(BufferCommand::FillBuffer {
            src: buffer,
            offset,
            size,
            data: 0,
        });
        self.push_buffer_access(BufferAccess {
            buffer,
            mutable: true,
            size,
            offset,
        });
    }

    // Update the buffer using memory that is directly stored within the command buffer
    pub unsafe fn cmd_update_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
        data: Vec<u8>,
    ) {
        self.push_buffer_cmd(BufferCommand::UpdateBuffer {
            src: buffer,
            offset,
            size,
            data,
        });
        self.push_buffer_access(BufferAccess {
            buffer,
            mutable: true,
            size,
            offset,
        });
    }
}

impl super::InsertVkCommand for BufferCommand {
    unsafe fn insert(
        self,
        device: &ash::Device,
        cmd: vk::CommandBuffer,
    ) {
        match self {
            BufferCommand::BindIndexBuffer {
                buffer,
                offset,
                index_type,
            } => device.cmd_bind_index_buffer(
                cmd, buffer, offset, index_type,
            ),
            BufferCommand::BindVertexBuffer {
                first_binding,
                buffers,
                offsets,
            } => device.cmd_bind_vertex_buffers(
                cmd,
                first_binding,
                &buffers,
                &offsets,
            ),
            BufferCommand::CopyBuffer { src, dst, regions } => {
                device.cmd_copy_buffer(cmd, src, dst, &regions)
            }
            BufferCommand::CopyImageToBuffer {
                dst,
                src,
                layout,
                regions,
            } => device.cmd_copy_image_to_buffer(
                cmd, src, layout, dst, &regions,
            ),
            BufferCommand::FillBuffer {
                src,
                offset,
                size,
                data,
            } => device.cmd_fill_buffer(cmd, src, offset, size, data),
            BufferCommand::UpdateBuffer {
                src,
                offset,
                size: _,
                data,
            } => device.cmd_update_buffer(cmd, src, offset, &data),
        }
    }
}
