use crate::{FrameRateLimit, Recorder, Submission};
use super::WindowSettings;
use bytemuck::{Pod, Zeroable};
use log_err::LogErrResult;
use parking_lot::Mutex;
use vulkano::{instance::{Instance, InstanceCreateInfo}, device::{physical::PhysicalDevice, Device, Queue}, VulkanLibrary, swapchain::Swapchain, image::SwapchainImage, memory::allocator::{GenericMemoryAllocator, StandardMemoryAllocator}, command_buffer::{allocator::{CommandBufferAllocator, StandardCommandBufferAlloc, StandardCommandBufferAllocator}, PrimaryAutoCommandBuffer, AutoCommandBufferBuilder}, sync::GpuFuture};
use std::sync::Arc;
use utils::ThreadPool;

// Graphical context that we will wrap around the WGPU instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics {
    // Main vulkan instance entry point
    pub(crate) instance: Arc<Instance>,

    // Physical graphics card that will crate the logical one
    pub(crate) physical: Arc<PhysicalDevice>,

    // Main logical device that will handle command and submitions
    pub(crate) device: Arc<Device>,

    // The graphics and presentable queue
    pub(crate) queue: Arc<Queue>,

    // Swapchain that we must render to and it's images
    pub(crate) swapchain: Arc<Swapchain>,
    pub(crate) images: Vec<Arc<SwapchainImage>>,
    pub(crate) recreate: Arc<Mutex<bool>>,

    // Allocator types
    pub(crate) memory_allocator: Arc<StandardMemoryAllocator>,
    pub(crate) cmd_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}

impl Graphics {    
    // Create a command recorder that will cache command buffer builders
    pub fn acquire(&self) -> Recorder {
        let builder = AutoCommandBufferBuilder::primary(
            self.cmd_buffer_allocator(),
            self.queue().queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::SimultaneousUse
        ).log_unwrap();

        Recorder {
            internal: builder,
        }
    }

    // Submit a command buffer recorder for execution
    // This will internally cached the built command buffer
    pub fn submit(&self, recorder: Recorder) -> Submission {
        let fence = vulkano::sync::now(self.device.clone())
            .then_execute(self.queue.clone(), recorder.internal.build().unwrap())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        fence.wait(None).unwrap();

        Submission {

        }
    }
}

impl Graphics {
    // Get the instance
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    // Get the adapter (physical device)
    pub fn physical(&self) -> &PhysicalDevice {
        &self.physical
    }

    // Get the logical device
    pub fn logical(&self) -> &Device {
        &self.device
    }

    // Get the main graphics + present queue
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    // Get the swapchain and it's images
    pub fn swapchain(&self) -> (&Swapchain, &[Arc<SwapchainImage>]) {
        (&self.swapchain, self.images.as_slice())
    }

    // Get the memory allocator
    pub fn memory_allocator(&self) -> &StandardMemoryAllocator {
        &self.memory_allocator
    }

    // Get the command buffer allocator
    pub fn cmd_buffer_allocator(&self) -> &StandardCommandBufferAllocator {
        &self.cmd_buffer_allocator
    }

}