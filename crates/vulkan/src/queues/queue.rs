use ash::vk;

// Queue that we can submit work to
pub struct Queue {
    flags: vk::QueueFlags,
    raw: vk::Queue,
}
