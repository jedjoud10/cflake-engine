use crate::Device;
use ash::vk;
use parking_lot::Mutex;
use utils::BitSet;

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
    // Each thread will have it's own command pool that it can acces and submit to

    // If a thread notices that a pool is not in use (no locks) it will aquire the lock
    // And start submitting to the pool as if it were it's own

    // If a thread does not find a free pool, it will simply allocate a new one for itself
    pub(super) pools: Vec<Pool>,
    pub(crate) free: Mutex<BitSet>,

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
        _index: usize,
    ) -> Option<&Pool> {
        todo!()
    }

    // Get a free pool that we can use directly
    pub fn aquire_pool(
        &self,
        _device: &Device,
        _flags: vk::CommandPoolCreateFlags,
    ) -> &Pool {
        todo!()
    }

    // Unlock a specific pool and return it to the family
    pub fn unlock_pool(&self, _pool: &Pool) {}
}

impl Family {
    // Create a new pool inside this family
    pub unsafe fn insert_new_pool(
        &self,
        device: &Device,
        flags: vk::CommandPoolCreateFlags,
    ) -> &Pool {
        let command_pool_create_info =
            vk::CommandPoolCreateInfo::builder()
                .flags(flags)
                .queue_family_index(self.family_index);

        // Create the command pool
        let _alloc = device
            .device
            .create_command_pool(&command_pool_create_info, None)
            .unwrap();
        log::debug!(
            "Inserted new pool inside family of index {}",
            self.family_index
        );

        /*
        // Create the pool
        let mut pools = self.pools.lock();
        let mut bitset = self.free.lock();

        // Insert the pool and set it to be free
        let pool = Pool::new(pools.len(), self.queue, alloc);
        let arc = Arc::new(pool);
        bitset.set(pools.len());
        pools.push(arc.clone());
        */
        todo!()
    }
}
