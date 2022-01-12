// Some pipeline commands
pub mod pipec {
    use crate::{
        object::{ObjectID, PipelineObject, PipelineTask, TaskID},
        pipeline::sender,
        Buildable, Pipeline,
    };

    // Send a task to the shared pipeline
    pub fn task(task: PipelineTask, pipeline: &Pipeline) -> TaskID {
        // Create a new task ID
        let id = TaskID::new(pipeline.task_statuses.get_next_idx_increment());
        // Get the thread local sender
        sender::send_task((task, id)).unwrap();
        id
    }
    // Create a Pipeline Object, returning it's ObjectID
    pub fn construct<T: PipelineObject + Buildable>(object: T, pipeline: &Pipeline) -> ObjectID<T> {
        let object = object.pre_construct(pipeline);
        // Construct it's ID and automatically send it's construction task
        let (t, id) = object.construct_task(pipeline);
        task(t, pipeline);
        id
    }
    // Create a Pipeline Object, but also return it's TaskID, so we can detect whenever the task has executed
    pub fn construct_return_task<T: PipelineObject + Buildable>(object: T, pipeline: &Pipeline) -> (TaskID, ObjectID<T>) {
        let object = object.pre_construct(pipeline);
        // Construct it's ID and automatically send it's construction task
        let (t, id) = object.construct_task(pipeline);

        (task(t, pipeline), id)
    }
    // Detect if a task has executed. If this task did indeed execute, it would be deleted next frame
    pub fn has_task_executed(id: TaskID, pipeline: &Pipeline) -> bool {
        pipeline.last_frame_task_statuses.contains(&id.index)
    }
}
