use std::ffi::CString;

use crate::{Adapter, Instance, Queues};
use ash::vk::{self, DeviceCreateInfo, DeviceQueueCreateInfo};

use gpu_allocator::{
    vulkan::{
        Allocation, AllocationCreateDesc, Allocator,
        AllocatorCreateDesc,
    },
    MemoryLocation,
};
use parking_lot::Mutex;
use utils::ThreadPool;

impl super::Device {
    // Create a single simple semaphore
    pub unsafe fn create_semaphore(&self) -> vk::Semaphore {
        self.device
            .create_semaphore(&Default::default(), None)
            .unwrap()
    }

    // Create a single simple fence
    pub unsafe fn create_fence(&self) -> vk::Fence {
        self.device.create_fence(&Default::default(), None).unwrap()
    }

    // Create raw buffer with no memory
    pub unsafe fn create_buffer(
        &self,
        size: u64,
        usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
        queues: &Queues,
    ) -> (vk::Buffer, Allocation) {
        // Setup vulkan info
        //let graphics = queues.family(FamilyType::Graphics);
        //let indices = [graphics.index()];
        let vk_info = vk::BufferCreateInfo::builder()
            .size(size)
            .flags(vk::BufferCreateFlags::empty())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            //.queue_family_indices(&indices)
            .usage(usage);

        // Create the buffer and fetch requirements
        log::debug!(
            "Creating buffer with size {} and usage {:?}",
            size,
            usage
        );
        let buffer =
            self.device.create_buffer(&vk_info, None).unwrap();

        // Get memory requirements
        log::debug!("Creating buffer memory for buffer {:?}", buffer);
        let requirements =
            self.device.get_buffer_memory_requirements(buffer);

        // Create gpu-allocator allocation
        let allocation = self
            .allocator
            .lock()
            .allocate(&AllocationCreateDesc {
                name: "",
                requirements,
                location,
                linear: true,
            })
            .unwrap();

        // Bind memory to the buffer
        unsafe {
            self.device
                .bind_buffer_memory(
                    buffer,
                    allocation.memory(),
                    allocation.offset(),
                )
                .unwrap()
        };

        // Create the tuple and return it
        (buffer, allocation)
    }

    // Get the device address of a buffer
    pub unsafe fn buffer_device_address(
        &self,
        buffer: vk::Buffer,
    ) -> vk::DeviceAddress {
        let builder =
            vk::BufferDeviceAddressInfo::builder().buffer(buffer);
        self.device.get_buffer_device_address(&*builder)
    }

    // Free a buffer and it's allocation
    pub unsafe fn destroy_buffer(
        &self,
        buffer: vk::Buffer,
        allocation: Allocation,
    ) {
        // Deallocate the underlying memory
        log::debug!(
            "Freeing allocation {:?}",
            allocation.mapped_ptr()
        );
        self.allocator.lock().free(allocation).unwrap();

        // Delete the Vulkan buffer
        let buffer = buffer;
        log::debug!("Freeing buffer {:?}", buffer);
        self.device.destroy_buffer(buffer, None);
    }
}
