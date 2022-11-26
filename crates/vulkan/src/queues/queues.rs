use std::{marker::PhantomData, cell::RefCell, sync::Arc, collections::HashMap};
use crate::{Adapter, Instance, Surface, Device};
use ash::vk;
use parking_lot::{Mutex, RwLock};
use super::family::{Family, FamilyType};

// Queues families and their queues that will be used by the logical device
pub struct Queues {
    families: Vec<Family>,
    graphics: usize,
    present: usize,
}

impl Queues {
    // Get the required queues from a logical device
    pub unsafe fn new(
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
        let present = Self::pick_queue_family(
            &families_properties,
            surface,
            adapter,
            true,
            vk::QueueFlags::empty(),
        );

        // Get the graphics queue family
        let graphics = Self::pick_queue_family(
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
                    queue: vk::Queue::null(),
                    pools: Vec::new(),
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
    pub(crate) unsafe fn complete_queue_creation(
        &mut self,
        device: &Device,
    ) {
        // Get the graphics family index
        let graphics = self.family_mut(FamilyType::Graphics);

        // Create the multiple command pools for multithreaded use only for the graphics family
        // TODO: Fix this and dynmacially allocate thread pools if needed
        for _ in 0..64 {
            graphics.insert_new_pool(device, Default::default());
        }

        // Get the present family index and create ONE pool for it (single threaded present)
        let present = self.family_mut(FamilyType::Present);
        present.insert_new_pool(device, Default::default())
    }

    // Find a queue that supports the specific flags
    unsafe fn pick_queue_family(
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

    // Destroy all pools and command buffers
    pub unsafe fn destroy(self, device: &Device) {
        device.device.device_wait_idle().unwrap();
        for family in self.families {
            for pool in family.pools {
                device.device.free_command_buffers(pool.alloc, pool.buffers.lock().as_slice());
                device.device.destroy_command_pool(pool.alloc, None);
            } 
        }
    }
}

impl Queues {
    // Get a family immutably using it's type
    pub fn family(&self, _type: FamilyType) -> &Family {
        &self.families[match _type {
            FamilyType::Graphics => self.graphics,
            FamilyType::Present => self.present,
        }]
    }

    // Get a family mutably using it's type
    pub fn family_mut(&mut self, _type: FamilyType) -> &mut Family {
        &mut self.families[match _type {
            FamilyType::Graphics => self.graphics,
            FamilyType::Present => self.present,
        }]
    }

    // Iterate over all the families immutably
    pub fn families(&self) -> impl Iterator<Item = &Family> {
        self.families.iter()
    }
    
    // Iterate over all the families mutably
    pub fn families_mut(&mut self) -> impl Iterator<Item = &mut Family> {
        self.families.iter_mut()
    }
}