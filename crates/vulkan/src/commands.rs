use super::Recorder;
use ahash::AHashMap;
use ash::vk;


// Recorder state that is stored within the recorders that is dynamically bound to command buffers
#[derive(Default)]
pub(crate) struct State {
    pub(crate) commands: Vec<Command>,
    pub(crate) access: Vec<(Access, usize)>,
}

// A finished command buffer state is what allows us to directly record Vulkan commands
pub(crate) struct CompletedState {
    pub(crate) groups: Vec<(Vec<Command>, Vec<Barrier>)> 
}

// Command pipeline barrier abstraction
// This helps automatically synchronizing vulkan commands
#[derive(Debug)]
pub(crate) struct Barrier {
    pub(crate) dependency_flags: vk::DependencyFlags,
    pub(crate) memory_barriers: Vec<vk::MemoryBarrier2>,
    pub(crate) buffer_memory_barriers: Vec<vk::BufferMemoryBarrier2>,
    pub(crate) image_memory_barriers: Vec<vk::ImageMemoryBarrier2>,
}

impl Barrier {
    // Combine two barriers together. This will result in a coarser barrier
    pub(crate) fn combine(&mut self, other: Self) {
        self.memory_barriers.extend(other.memory_barriers);
        self.buffer_memory_barriers.extend(other.buffer_memory_barriers);
        self.image_memory_barriers.extend(other.image_memory_barriers);
        self.dependency_flags |= other.dependency_flags;
    }   
}

// Any type of command that can be applied
#[derive(Debug)]
pub(crate) enum Command {
    Buffer(BufferCommand),
}

// Any type of command access
pub(crate) enum Access {
    Buffer(BufferAccess),
}

impl CompletedState {
    // Execute the completed / finished commands that we have executed
    // This will not be executed each time we submit the buffer for performance reasons
    pub(crate) unsafe fn insert(
        self,
        device: &ash::Device,
        cmd: vk::CommandBuffer,
    ) {
        for group in self.groups {
            let commands = group.0;
            let barriers = group.1;
            //log::debug!("Executing completed stage group with {} commands and barrier: {}", commands.len(), barrier.is_some());


            for barrier in barriers {
                device.cmd_pipeline_barrier2(
                    cmd,
                    &*vk::DependencyInfo::builder()
                        .dependency_flags(barrier.dependency_flags)
                        .memory_barriers(&barrier.memory_barriers)
                        .buffer_memory_barriers(&barrier.buffer_memory_barriers)
                        .image_memory_barriers(&barrier.image_memory_barriers)
                );
            }
            
            for command in commands {
                dbg!(&command);
                match command {
                    Command::Buffer(command) => command.insert(device, cmd),
                }
            }
        }
    }
}

// Enum that contains all the types of commands that can be applied to buffers
#[derive(Debug)]
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
        data: Vec<u8>,
    },
}

// Contains an access window for a specific buffer
pub(crate) struct BufferAccess {
    pub(crate) flags: vk::AccessFlags2,
    pub(crate) stage: vk::PipelineStageFlags2,
    pub(crate) buffer: vk::Buffer,
    pub(crate) mutable: bool,
    pub(crate) size: u64,
    pub(crate) offset: u64,
}

// Buffer commands
impl Recorder {
    // Add a new buffer command internally
    unsafe fn push_buffer_cmd(&mut self, cmd: BufferCommand, access: impl IntoIterator<Item = BufferAccess>) {
        let index = self.state.commands.len();
        self.state.commands.push(Command::Buffer(cmd));
        self.state.access.extend(access.into_iter().map(|a| (Access::Buffer(a), index)));
    }

    // Bind an index buffer to the command buffer render pass
    pub unsafe fn cmd_bind_index_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
        self.push_buffer_cmd(BufferCommand::BindIndexBuffer {
            buffer,
            offset,
            index_type,
        }, Some(BufferAccess {
            flags: vk::AccessFlags2::INDEX_READ, 
            stage: vk::PipelineStageFlags2::INDEX_INPUT,
            buffer,
            mutable: false,
            size: vk::WHOLE_SIZE,
            offset,
        }));
    }

    // Bind vertex buffers to the command buffer render pass
    pub unsafe fn cmd_bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers: Vec<vk::Buffer>,
        offsets: Vec<vk::DeviceSize>,
    ) {
        self.push_buffer_cmd(BufferCommand::BindVertexBuffer {
            first_binding,
            buffers: buffers.clone(),
            offsets: offsets.clone(),
        }, buffers.into_iter().enumerate().map(|(i, buffer)| BufferAccess {
            flags: vk::AccessFlags2::VERTEX_ATTRIBUTE_READ, 
            stage: vk::PipelineStageFlags2::VERTEX_ATTRIBUTE_INPUT,
            buffer,
            mutable: false,
            size: vk::WHOLE_SIZE,
            offset: offsets[i],
        }));
    }

    // Copy a buffer to another buffer in GPU memory
    pub unsafe fn cmd_copy_buffer(
        &mut self,
        src: vk::Buffer,
        dst: vk::Buffer,
        regions: Vec<vk::BufferCopy>,
    ) {
        self.push_buffer_cmd(BufferCommand::CopyBuffer {
            src,
            dst,
            regions: regions.clone(),
        }, regions.into_iter().flat_map(|buffer_copy| vec![BufferAccess {
            flags: vk::AccessFlags2::TRANSFER_READ, 
            stage: vk::PipelineStageFlags2::TRANSFER,
            buffer: src,
            mutable: false,
            size: buffer_copy.size,
            offset: buffer_copy.src_offset,
        }, BufferAccess {
            flags: vk::AccessFlags2::TRANSFER_WRITE, 
            stage: vk::PipelineStageFlags2::TRANSFER,
            buffer: dst,
            mutable: true,
            size: buffer_copy.size,
            offset: buffer_copy.dst_offset,
        }]));
    }

    // Copy an image to a buffer in GPU memory
    pub unsafe fn cmd_copy_image_to_buffer(
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
        }, None);
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
        }, Some(BufferAccess {
            flags: vk::AccessFlags2::TRANSFER_WRITE,
            stage: vk::PipelineStageFlags2::TRANSFER,
            buffer,
            mutable: true,
            size,
            offset,
        }));
    }

    // Update the buffer using memory that is directly stored within the command buffer
    pub unsafe fn cmd_update_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        data: Vec<u8>,
    ) {
        let size = data.len() as u64;
        self.push_buffer_cmd(BufferCommand::UpdateBuffer {
            src: buffer,
            offset,
            data,
        }, Some(BufferAccess {
            flags: vk::AccessFlags2::TRANSFER_WRITE,
            stage: vk::PipelineStageFlags2::TRANSFER,
            buffer,
            mutable: true,
            size,
            offset,
        }));
    }
}

impl BufferCommand {
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
                data,
            } => device.cmd_update_buffer(cmd, src, offset, &data),
        }
    }
}
