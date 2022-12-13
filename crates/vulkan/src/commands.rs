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
    dependency_flags: vk::DependencyFlags,
    memory_barriers: Vec<vk::MemoryBarrier2>,
    buffer_memory_barriers: Vec<vk::BufferMemoryBarrier2>,
    image_memory_barriers: Vec<vk::ImageMemoryBarrier2>,
}

impl Barrier {
    // Combine two barriers together. This will result in a coarser barrier
    fn combine(mut self, other: Self) -> Barrier {
        self.memory_barriers.extend(other.memory_barriers);
        self.buffer_memory_barriers.extend(other.buffer_memory_barriers);
        self.image_memory_barriers.extend(other.image_memory_barriers);
        
        Self {
            dependency_flags: self.dependency_flags | other.dependency_flags,
            memory_barriers: self.memory_barriers,
            buffer_memory_barriers: self.buffer_memory_barriers,
            image_memory_barriers: self.image_memory_barriers,
        }
    }   
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
                device.cmd_pipeline_barrier2(
                    cmd,
                    &*vk::DependencyInfo::builder()
                        .dependency_flags(barrier.dependency_flags)
                        .memory_barriers(&barrier.memory_barriers)
                        .buffer_memory_barriers(&barrier.buffer_memory_barriers)
                        .image_memory_barriers(&barrier.image_memory_barriers)
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
    flags: vk::AccessFlags2,
    stage: vk::PipelineStageFlags2,
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
        let index = self.state.commands.len();
        self.state.commands.push(Command::Buffer(cmd));
        self.state.access.extend(access.into_iter().map(|a| (Access::Buffer(a), index)));
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
            flags: vk::AccessFlags2::INDEX_READ, 
            stage: vk::PipelineStageFlags2::INDEX_INPUT,
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
            flags: vk::AccessFlags2::VERTEX_ATTRIBUTE_READ, 
            stage: vk::PipelineStageFlags2::VERTEX_ATTRIBUTE_INPUT,
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
        log::debug!("Commands::copy_buffer, src = {:?}, dst = {:?}", src, dst);
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
        size: vk::DeviceSize,
        data: Vec<u8>,
    ) {
        self.push_buffer_cmd(BufferCommand::UpdateBuffer {
            src: buffer,
            offset,
            size,
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
                size: _,
                data,
            } => device.cmd_update_buffer(cmd, src, offset, &data),
        }
    }
}

// proto-barrier for mut access of buffer2
// proto-barrier for ref access of buffer1
// vkCopyBuffer(src = buffer1, dst = buffer2)
// vkDispatch
// vkCopyImageToBuffer(src = image1, dst = buffer 3)

// proto-barrier for mut access of buffer1
// proto-barrier for ref access of buffer2
// vkCopyBuffer(src = buffer2, dst = buffer1)

// Create a prototype barrier for a specific access BEFORE the command
fn prototype(access: &Access, states: &mut AHashMap::<vk::Buffer, (vk::AccessFlags2, vk::PipelineStageFlags2)>) -> Barrier {
    match access {
        Access::Buffer(buffer) => {
            // Get the old state and access of the buffer
            //let (old_access_flags, old_stage_flags) = states.get(&buffer.buffer).unwrap();
            let BufferAccess { flags, stage, buffer, mutable, size, offset } = buffer;
            log::warn!("{:?}", &*buffer);

            let barrier = vk::BufferMemoryBarrier2::builder()
                .buffer(*buffer)
                .dst_access_mask(*flags)
                .dst_stage_mask(*stage);

            let barrier = if let Some((flags, state)) = states.get(&buffer) {
                barrier.src_access_mask(*flags).src_stage_mask(*state)
            } else {
                barrier
            };

            dbg!(&*barrier);
            states.insert(*buffer, (*flags, *stage));

            Barrier {
                dependency_flags: vk::DependencyFlags::empty(),
                memory_barriers: Default::default(),
                buffer_memory_barriers: vec![*barrier],
                image_memory_barriers: Default::default(),
            }
        },
    }
}

// Convert the locally stored command to local groups that automatically place barriers within them
pub(super) fn complete(state: State) -> CompletedState {
    // Keep track of what buffers have mutable access and what stage needs mutable access
    let mut exclusive_buffer_access = AHashMap::<vk::Buffer, (vk::AccessFlags2, vk::PipelineStageFlags2)>::new();

    // Pipeline barrier that must be placed before commands
    let mut commands_test = AHashMap::<usize, Barrier>::new();

    // Create a prototype barrier for each access
    for (access, command) in &state.access {
        let other = prototype(access, &mut exclusive_buffer_access);

        commands_test.entry(*command);
    }

    todo!()



    /*
    let commands = state
        .commands
        .into_iter()
        .enumerate()
        .map(|(i, command)| (vec![command], Some());

        


    CompletedState {
        groups: commands.collect(),
    }
    */
}