use std::sync::Arc;

use ash::vk;
use parking_lot::{Mutex, MutexGuard};
use utils::BitSet;

use crate::{State, Device};

// Abstraction around a Vulkan command pool
pub struct Pool {
    // Underlying pool
    pub(crate) pool: vk::CommandPool,

    // All the buffers that we allocated
    pub(crate) buffers: Mutex<Vec<vk::CommandBuffer>>,

    // Paused command buffer states
    pub(crate) states: Arc<Mutex<Vec<State>>>,

    // Number of command buffers that are currently recording
    pub(crate) free: Arc<Mutex<BitSet>>,
}

impl Pool {
    // Create a new pool for a specific thread
    pub(super) unsafe fn new(device: &Device, qfi: u32, flags: vk::CommandPoolCreateFlags) -> Self {
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(flags)
            .queue_family_index(qfi);
        let pool = device.device.create_command_pool(&info, None).unwrap();
        Self {
            pool,
            buffers: Mutex::new(Vec::new()),
            states: Arc::new(Mutex::new(Vec::new())),
            free: Arc::new(Mutex::new(BitSet::new())),
        }
    }

    // Allocate some new command buffers
    pub(super) unsafe fn allocate_cmd_buffers(
        &self,
        device: &Device,
        count: usize,
    ) -> Vec<(usize, vk::CommandBuffer)> {
        let mut bitset = self.free.lock();
        let mut states = self.states.lock();
        let mut buffers = self.buffers.lock();
        let offset = buffers.len();

        // Set the "free" bit for each command buffer
        for i in offset..(offset+count) {
            bitset.set(i);
        }

        // Setup the allocation info
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(count as u32)
            .command_pool(self.pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        // Add the new command buffers
        let new_buffers = device
            .device
            .allocate_command_buffers(&allocate_info)
            .unwrap();
        buffers.extend(new_buffers);

        // Add the new command buffer states
        let new_states = (0..count)
            .into_iter()
            .map(|_| State::default());
        states.extend(new_states);

        // Fetch the command buffers now
        buffers[offset..]
            .iter()
            .cloned()
            .enumerate()
            .collect::<Vec<_>>()
    }

    // Aquire a command buffer that is in the recording or reset state
    pub(super) unsafe fn aquire_cmd_buffer(
        &self,
        device: &Device,
        allocate: bool,
    ) -> (usize, vk::CommandBuffer) {
        // Try to find a free command buffer
        if let Some(free) = self.free.lock().find_one_from(0) {
            let buffers = self.buffers.lock();
            todo!()
        } else if allocate {
            self.allocate_cmd_buffers(device, 1)[0]
        } else {
            log::error!("Could not find a free command buffer");
            panic!();
        }        
    }
}
