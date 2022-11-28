use crate::Device;
use ash::vk;
use parking_lot::Mutex;
use utils::{BitSet, ImmutableVec};

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
    pub(super) pools: ImmutableVec<Pool>,
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
        index: usize,
    ) -> Option<&Pool> {
        self.pools.get(index)
    }

    // Get a free pool that we can use directly
    pub fn aquire_pool(
        &self,
        device: &Device,
        flags: vk::CommandPoolCreateFlags,
    ) -> &Pool {
        // Check if we have a free pool
        let mut bitset = self.free.lock();
        if let Some(index) = bitset.find_one_from(0) {
            bitset.remove(index);
            log::debug!("Found a free command pool at index {index}");
            self.pools.get(index).unwrap()
        } else {
            // Allocate a new pool
            log::debug!("Could not find free command pool, allocating a new one");
            drop(bitset);
            unsafe { self.insert_new_pool(device, flags) }
        }
    }

    // Unlock a specific pool and return it to the family
    pub fn unlock_pool(&self, pool: &Pool) {
        let mut bitset = self.free.lock();
        bitset.set(pool.index);
    }
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
        let mut bitset = self.free.lock();
        let pool = Pool::new(self.pools.len(), flags, self.queue, alloc);
        bitset.set(self.pools.len());
        self.pools.push(pool);
        self.pools.last().unwrap()
    }
}
