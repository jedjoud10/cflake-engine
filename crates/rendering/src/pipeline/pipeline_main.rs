// Some pipeline commands
pub mod pipec {
    use others::TaskSender;

    use crate::{SharedPipeline, object::PipelineTask};

    // Send a builder to the render thread so it can build it's inner value
    pub fn construct(pipeline: &SharedPipeline, sender: TaskSender<PipelineTask>) {
        
    }
}