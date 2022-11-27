use ash::vk;
use parking_lot::Mutex;
use crate::{Device};

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

    // Current command pools of this family (multiple command pools per family)
    // TODO: Dynamic pools?
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

    // Get the pool for the current thread immutably
    pub fn pool(&self) -> &Pool {
        &self.pools[0]
    }

    // Get the pool for the current thread mutably
    pub fn pool_mut(&mut self) -> &mut Pool {
        &mut self.pools[0]
    }

    // Get a free pool that we can use directly
    pub fn aquire_pool(&self) -> &Pool {
        // TODO: actually write this
        &self.pools[0]
    }
}

impl Family {
    // Create a new pool inside this family
    pub unsafe fn insert_new_pool(&mut self, device: &Device, flags: vk::CommandPoolCreateFlags) {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(flags)
            .queue_family_index(self.family_index);

        // Create the command pool
        let alloc = device.device.create_command_pool(
            &command_pool_create_info,
            None
        ).unwrap();
        log::debug!("Inserted new pool inside family of index {}", self.family_index);

        // Create the pool
        let pool = Pool { 
            alloc,
            buffers: Mutex::new(Vec::new()),
            queue: self.queue
        };

        // Allocate a singular command buffer
        pool.allocate_command_buffers(device, 1, false);

        self.pools.push(pool);
    }
}