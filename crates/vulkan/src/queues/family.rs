use crate::Device;
use ash::vk;
use utils::ThreadPool;

use super::pool::Pool;

// Family type for graphics or present family
#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub enum FamilyType {
    // The graphics family will be responsible of handling graphics command
    Graphics,

    // The present family will be responsible of presenting the images to the screen
    Present,
}

// A single queue family abstraction
pub struct Family {
    // Queue flags of the current family
    pub(super) family_queue_flags: vk::QueueFlags,
    pub(super) family_index: u32,

    // One command pool per thread
    pub(super) pools: Vec<Pool>,

    // TODO: We should be able to have multiple queues per family but mkay
    pub(super) queue: vk::Queue,
}

impl Family {
    // Get the queue handle
    pub fn queue(&self) -> vk::Queue {
        self.queue
    }

    // Get the family index
    pub fn index(&self) -> u32 {
        self.family_index
    }

    // Get a specific pool, even though it is currently in use
    // This will never create a new pool if needed
    pub unsafe fn aquire_specific_pool(
        &self,
        index: usize,
    ) -> Option<&Pool> {
        self.pools.get(index)
    }

    // Get the command pool for the current thread
    pub fn aquire_pool(
        &self,
    ) -> &Pool {
        &self.pools[ThreadPool::current()]
    }
}

impl Family {
    // Create a new pool inside this family
    pub unsafe fn insert_new_pool(
        &mut self,
        device: &Device,
        flags: vk::CommandPoolCreateFlags,
    ) -> &Pool {
        let command_pool_create_info =
            vk::CommandPoolCreateInfo::builder()
                .flags(flags)
                .queue_family_index(self.family_index);

        // Create the command pool
        let alloc = device
            .device
            .create_command_pool(&command_pool_create_info, None)
            .unwrap();
        device.device.device_wait_idle().unwrap();
        log::warn!(
            "Inserted new pool inside family of index {}",
            self.family_index
        );

        // Insert the pool and set it to be free
        let pool = Pool::new(self.pools.len(), flags, self.queue, alloc);
        self.pools.push(pool);
        self.pools.last().unwrap()
    }
}
