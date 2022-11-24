use crate::{Adapter, Instance, Surface, Device};
use ash::vk;
use world::ThreadPool;

// A single command pool abstraction
// We technically should have one pool per thread
pub(crate) struct Pool {
    // Raw vulkan command pool
    pub(crate) alloc: vk::CommandPool,
    
    // All the command buffers allocated in this command pool
    pub(crate) buffers: Vec<vk::CommandBuffer>,
}

// A single queue family abstraction
pub(crate) struct Family {
    // Queue flags of the current family
    pub(crate) family_queue_flags: vk::QueueFlags,
    pub(crate) family_index: u32,

    // Current command pools of this family (multiple command pools per family)
    pub(crate) pools: Vec<Pool>,

    // TODO: We should be able to have multiple queues per family but mkay
    pub(crate) queue: vk::Queue,
}

// Queues and their families that will be used by the logical device
pub(crate) struct Queues {
    pub(crate) families: Vec<Family>,
    pub(crate) graphics: usize,
    pub(crate) present: usize,
} 

impl Queues {
    // Destroy all pools and command buffers
    pub(crate) unsafe fn destroy(self, device: &Device) {
        device.device.device_wait_idle().unwrap();
        for family in self.families {
            for pool in family.pools {
                device.device.free_command_buffers(pool.alloc, pool.buffers.as_slice());
                device.device.destroy_command_pool(pool.alloc, None);
            } 
        }
    }
}

// Get the required queues from a logical device
pub(crate) unsafe fn create_queues(
    adapter: &Adapter,
    surface: &Surface,
    instance: &Instance,
) -> Queues {
    let families_properties = instance
        .instance
        .get_physical_device_queue_family_properties(
            adapter.physical_device,
        );

    // Get the present queue family
    let present = pick_queue_family(
        &families_properties,
        surface,
        adapter,
        true,
        vk::QueueFlags::empty(),
    );

    // Get the graphics queue family
    let graphics = pick_queue_family(
        &families_properties,
        surface,
        adapter,
        false,
        vk::QueueFlags::GRAPHICS,
    );

    // Convert to vector
    let mut families = vec![present, graphics];
    families.sort_unstable();
    families.dedup();

    // Find indices AGAIN
    let graphics = families.iter().position(|&i| i == graphics).unwrap();
    let present = families.iter().position(|&i| i == present).unwrap();

    // Create placeholder families
    let families = families
        .into_iter()
        .map(|i| {
            // Get the family queue flags again
            let flags = families_properties[i as usize].queue_flags;

            // Create placeholder family value
            Family {
                family_queue_flags: flags,
                family_index: i,
                pools: Vec::new(),
                queue: vk::Queue::null(),
            }
        })
        .collect::<Vec<_>>();
    Queues {
        families,
        graphics,
        present,
    }
}

// Update the queues after we've made the device
pub(super) unsafe fn complete_queue_creation(
    device: &Device,
    queues: &mut Queues
) {
    // Get the graphics family index
    let graphics = &mut queues.families[queues.graphics];

    // Create the multiple command pools for multithreaded use only for the graphics family
    // TODO: Fix this and dynmacially allocate thread pools if needed
    let pools = (0..ThreadPool::default_thread_count())
        .into_iter()
        .map(|_| {
            // Create a new command pool create info
            let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::empty())
                .queue_family_index(graphics.family_index);

            // Create the command pool
            let alloc = device.device.create_command_pool(
                &command_pool_create_info,
                None
            ).unwrap();

            // Structify
            Pool {
                alloc,
                buffers: Default::default(),
            }
        })
        .collect();
    graphics.pools = pools;    
}

// Find a queue that supports the specific flags
pub(super) unsafe fn pick_queue_family(
    family_properties: &[vk::QueueFamilyProperties],
    surface: &Surface,
    adapter: &Adapter,
    supports_presenting: bool,
    flags: vk::QueueFlags,
) -> u32 {
    family_properties
        .iter()
        .enumerate()
        .position(|(i, props)| {
            // Check if the queue family supporsts the flags
            let flags = props.queue_flags.contains(flags);

            // If the queue we must fetch must support presenting, fetch the physical device properties
            let presenting = !supports_presenting
                || surface
                    .surface_loader
                    .get_physical_device_surface_support(
                        adapter.physical_device,
                        i as u32,
                        surface.surface,
                    )
                    .unwrap();

            flags && presenting
        })
        .expect("Could not find the graphics queue") as u32
}
