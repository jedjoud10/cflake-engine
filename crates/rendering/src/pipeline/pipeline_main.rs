// Some pipeline commands
pub mod pipec {
    use std::sync::mpsc::Sender;
    use crate::{object::{PipelineTask, TaskID, PipelineObject, ObjectID}, Buildable, Pipeline};

    // Send a task to the shared pipeline 
    pub fn task(task: PipelineTask, pipeline: &Pipeline) -> TaskID {
        // Create a new task ID
        let id = TaskID::new(pipeline.task_statuses.get_next_idx_increment());
        pipeline.tx.send((task, id)).unwrap();
        id
    }
    // Create a Pipeline Object, returning it's ObjectID
    pub fn construct<T: PipelineObject + Buildable>(object: T, pipeline: &Pipeline) -> ObjectID<T> {
        let object = object.pre_construct(pipeline);
        // Construct it's ID and automatically send it's construction task
        object.construct(pipeline)
    }
}