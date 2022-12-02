use crate::{Family, Pool, Queue, Recorder, Adapter, Instance, Device};
use ash::vk;
use smallvec::SmallVec;

// Queues families and their queues that will be used by the logical device
// Even though this is named "Queues", it doesn't contains the queues directly
pub struct Queues(Vec<Family>);

impl Queues {
    // Create the queue families, queues, and default pools
    pub unsafe fn new(instance: &Instance, device: &Device, adapter: &Adapter) -> Self {
        // Get the present queue family
        let present = Self::pick_queue_family(
            &adapter.queue_family_properties,
            adapter,
            true,
            vk::QueueFlags::empty(),
        );

        // Get the graphics queue family
        let graphics = Self::pick_queue_family(
            &adapter.queue_family_properties,
            adapter,
            false,
            vk::QueueFlags::GRAPHICS,
        );

        // Convert to vector
        let families = vec![present, graphics];

        // Create placeholder families
        let families = families
            .into_iter()
            .map(|qfi| {
                // Get the family queue flags again
                let properties = adapter
                    .queue_family_properties[i as usize];

                // Check if we can present to this queue
                let present = adapter
                    .queue_family_surface_supported[i as usize];

                // Create placeholder family value
                Family::new(device, qfi, properties, present, true)
            })
            .collect::<Vec<_>>();

        Queues(families)
    }


    // Find a queue that supports the specific flags
    pub(crate) unsafe fn pick_queue_family(
        family_properties: &[vk::QueueFamilyProperties],
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
                let presenting = !supports_presenting || adapter.queue_family_surface_supported[i];
                flags && presenting
            })
            .unwrap() as u32
    }
}

impl Queues {
    // Get the family indices even before we create the queues
    pub unsafe fn indices(adapter: &Adapter) -> Vec<u32> {
        // Get the present queue family
        let present = Self::pick_queue_family(
            &adapter.queue_family_properties,
            adapter,
            true,
            vk::QueueFlags::empty(),
        );
        
        // Get the graphics queue family
        let graphics = Self::pick_queue_family(
            &adapter.queue_family_properties,
            adapter,
            false,
            vk::QueueFlags::GRAPHICS,
        );

        // Return the indices
        let mut vec = vec![present, graphics];
        vec.dedup();
        return vec;
    }

    // Get a specific family of a specific type
    pub unsafe fn family(
        &self,
        flags: Option<vk::QueueFlags>,
        present: bool,
    ) -> &Family {
        self.0
            .iter()
            .find(|f| {
                flags.map(|flags| f.properties.queue_flags.contains(flags)).unwrap_or(true) && (!present || f.present)
            })
            .unwrap()
    }

    // Get a command recorder for a specific family
    pub unsafe fn aquire<'a>(
        &'a self,
        device: &'a Device,
        flags: Option<vk::QueueFlags>,
        present: bool,
        pool_create_flags: Option<vk::CommandPoolCreateFlags>,
        cmd_buffer_begin: vk::CommandBufferUsageFlags,
        allocate: bool,
    ) -> Recorder<'a> {
        // Get the appropriate family
        let family = self.family(flags, present);

        // Get a free command pool in that family
        let pool = family.aquire_pool(pool_create_flags);

        // Get a command buffer
        let (index, cmd) = pool.aquire_cmd_buffer(device, allocate);
        let state = pool.states.clone();
        let free = pool.free.clone();

        // Create the command buffer info
        let info = *vk::CommandBufferBeginInfo::builder()
            .flags(cmd_buffer_begin);

        Recorder {
            index,
            cmd,
            state,
            device,
            pool,
            info,
            free,
        }
    }
}
