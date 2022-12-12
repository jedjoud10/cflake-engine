use super::Recorder;
use ahash::AHashMap;
use ash::vk;


// Recorder state that is stored within the recorders that is dynamically bound to command buffers
#[derive(Default)]
pub(super) struct State {
    pub(super) commands: Vec<Command>,
    pub(super) access: Vec<(Access, usize)>,
}

// A finished command buffer state is what allows us to directly record Vulkan commands
pub(super) struct CompletedState {
    groups: Vec<(Vec<Command>, Option<Barrier>)> 
}

// Command pipeline barrier abstraction
// This helps automatically synchronizing vulkan commands
pub(super) struct Barrier {
    src_stage_mask: vk::PipelineStageFlags,
    dst_stage_mask: vk::PipelineStageFlags,
    dependency_flags: vk::DependencyFlags,
    memory_barriers: Vec<vk::MemoryBarrier>,
    buffer_memory_barriers: Vec<vk::BufferMemoryBarrier>,
    image_memory_barriers: Vec<vk::ImageMemoryBarrier>,
}

// Any type of command that can be applied
pub(super) enum Command {
    Buffer(BufferCommand),
}

// Any type of command access
pub(super) enum Access {
    Buffer(BufferAccess),
}

impl CompletedState {
    pub(super) unsafe fn insert(
        self,
        device: &ash::Device,
        cmd: vk::CommandBuffer,
    ) {
        for group in self.groups {
            let commands = group.0;
            let barrier = group.1;

            for command in commands {
                match command {
                    Command::Buffer(command) => command.insert(device, cmd),
                }
            }

            if let Some(barrier) = barrier {
                device.cmd_pipeline_barrier(
                    cmd,
                    barrier.src_stage_mask,
                    barrier.dst_stage_mask,
                    barrier.dependency_flags,
                    &barrier.memory_barriers,
                    &barrier.buffer_memory_barriers,
                    &barrier.image_memory_barriers,
                );
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
        size: vk::DeviceSize,
        data: Vec<u8>,
    },
}

// Enum that tells us how a buffer is accessed
pub(super) struct BufferAccess {
    buffer: vk::Buffer,
    mutable: bool,
    size: u64,
    offset: u64,
}

// Buffer commands
impl Recorder {
    // Add a new buffer command internally
    unsafe fn push_buffer_cmd(&mut self, cmd: BufferCommand, access: impl IntoIterator<Item = BufferAccess>) {
        log::debug!("Recorder::push_buffer_cmd {:?}", &cmd);
        self.state.commands.push(Command::Buffer(cmd));
        self.state.access.extend(access.into_iter().map(|a| (Access::Buffer(a), 0)));
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
        }, Some(BufferAccess {
            buffer,
            mutable: false,
            size: vk::WHOLE_SIZE,
            offset,
        }));
    }

    // Bind vertex buffers to the command buffer render pass
    pub unsafe fn bind_vertex_buffers(
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
            buffer,
            mutable: false,
            size: vk::WHOLE_SIZE,
            offset: offsets[i],
        }));
    }

    // Copy a buffer to another buffer in GPU memory
    pub unsafe fn copy_buffer(
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
            buffer: src,
            mutable: false,
            size: buffer_copy.size,
            offset: buffer_copy.src_offset,
        }, BufferAccess {
            buffer: dst,
            mutable: true,
            size: buffer_copy.size,
            offset: buffer_copy.dst_offset,
        }]));
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
        size: vk::DeviceSize,
        data: Vec<u8>,
    ) {
        self.push_buffer_cmd(BufferCommand::UpdateBuffer {
            src: buffer,
            offset,
            size,
            data,
        }, Some(BufferAccess {
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
                size: _,
                data,
            } => device.cmd_update_buffer(cmd, src, offset, &data),
        }
    }
}

// Convert the locally stored command to local groups that automatically place barriers within them
pub(super) fn complete(state: State) -> CompletedState {
    // Keep track of what buffers have mutable access and what stage needs mutable access
    let mut exclusive_buffer_access = AHashMap::<usize, vk::AccessFlags>::new();

    // Iterate over all the accesses
    for (access, index) in state.access {
        match access {
            Access::Buffer(_) => todo!(),
        }
    }


    let commands = state.commands;


    CompletedState {
        groups: vec![(commands, None)],
    }
}