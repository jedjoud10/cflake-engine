use ash::vk;
use smallvec::SmallVec;

use crate::Device;

// CPU Side commands that can sort through and remove redundent commands
pub(super) enum Command {
    // Copy multiple buffer regions to another buffer
    BufferCopy {
        src: vk::Buffer,
        dst: vk::Buffer,
        regions: Vec<vk::BufferCopy>,
    },

    // Fill a buffer with some data
    BufferFill {
        src: vk::Buffer,
        offset: u64,
        size: u64,
        data: u32,
    },
}

// Saved states that allow use to combine multiple recorders implicitly
#[derive(Default)]
pub(crate) struct State(pub(super) Vec<Command>);

impl State {
    // Sort the commands and remove redudent commands
    fn sort_and_merge(&mut self) {
        // Contains lists of batched commands
        let groups: Vec<Command> = Vec::new();
    }

    // Write a single command to a command buffer
    unsafe fn encode_command(
        command: Command,
        buffer: vk::CommandBuffer,
        device: &Device,
    ) {
        match command {
            Command::BufferCopy { src, dst, regions } => {
                device
                    .device
                    .cmd_copy_buffer(buffer, src, dst, &regions);
            }
            Command::BufferFill {
                src,
                offset,
                size,
                data,
            } => {
                device
                    .device
                    .cmd_fill_buffer(buffer, src, offset, size, data);
            }
        }
    }

    // Convert the CPU commands to actual vulkan commands and write them to the given buffer
    // This will begin recording the buffer and finish recording it immediately
    pub(crate) unsafe fn finish(
        mut self,
        buffer: vk::CommandBuffer,
        begin_info: vk::CommandBufferBeginInfo,
        device: &Device,
    ) {
        self.sort_and_merge();

        device
            .device
            .begin_command_buffer(buffer, &begin_info)
            .unwrap();

        for command in self.0 {
            Self::encode_command(command, buffer, device);
        }

        device.device.end_command_buffer(buffer).unwrap();
    }
}
