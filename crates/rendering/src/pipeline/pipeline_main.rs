// Some pipeline commands
pub mod pipec {
    use std::sync::mpsc::Sender;
    use crate::{object::{PipelineTask, TaskID, PipelineObject, ObjectID}, Pipeline, Buildable};

    // Send a task to the shared pipeline 
    pub fn task(task: PipelineTask, pipeline: &Pipeline) -> TaskID {
        todo!();
    }
    // Create a Pipeline Object, returning it's ObjectID
    pub fn construct<T: PipelineObject + Buildable>(object: T, pipeline: &Pipeline) -> ObjectID<T> {
        todo!();
    }
}