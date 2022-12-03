use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

// A recorder can keep a primary command buffer cached until we flush it
// This is used to reduce the number of submissions we have to make to the GPU
pub struct Recorder {
    pub internal: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
}

impl Recorder {

}

// This is a submission of a command recorder
// The underlying command buffer might've not been submitted yet
pub struct Submission {

}