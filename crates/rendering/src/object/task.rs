// A pipeline task that will be sent to the render thread
pub enum PipelineTask {
}

// The status for a specific PipelineTask
pub enum PipelineTaskStatus {
    Pending,
    Running,
    Finished,
}