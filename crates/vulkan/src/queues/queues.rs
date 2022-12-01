use crate::{Family, Pool, Queue, Recorder};
use ash::vk;
use smallvec::SmallVec;

// Queues families and their queues that will be used by the logical device
// Even though this is named "Queues", it doesn't contains the queues directly
pub struct Queues(Vec<Family>);

impl Queues {
    // Get a command recorder for a specific family
    pub unsafe fn aquire(
        &self,
        allocate: bool,
        pool: vk::CommandPoolCreateFlags,
        begin: vk::CommandBufferUsageFlags,
    ) -> Recorder {
        todo!()
    }

    // Return a command buffer to it's pool and pause it
    pub unsafe fn pause(&self, recorder: Recorder) {
        todo!()
    }
}
