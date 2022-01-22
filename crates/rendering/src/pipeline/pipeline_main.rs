// Some pipeline commands
pub mod pipec {
    use crate::{
        basics::Buildable,
        object::{ObjectID, PipelineObject, PipelineTask, TrackingTaskID},
        pipeline::{sender, Pipeline},
    };

    // Send a task to the shared pipeline
    pub fn task(task: PipelineTask, pipeline: &Pipeline) {
        sender::send_task((task, None), pipeline).unwrap();
    }
    // Send a task to the shared pipeline, but also return it's tracking ID
    pub fn task_tracker(task: PipelineTask, pipeline: &Pipeline) -> TrackingTaskID {
        // Create a tracking task ID for this task 
        let id = TrackingTaskID::new();
        // Get the thread local sender
        sender::send_task((task, Some(id)), pipeline).unwrap();
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
    // Create a Pipeline Object, returning it's ObjectID, but without running it's pre construct
    pub(crate) fn construct_only<T: PipelineObject + Buildable>(object: T, pipeline: &Pipeline) -> ObjectID<T> {
        // Construct it's ID and automatically send it's construction task
        let (t, id) = object.construct_task(pipeline);
        task(t, pipeline);
        id
    }
    // Create a Pipeline Object, but also return it's TrackingTaskID, so we can detect whenever the task has executed
    pub fn construct_return_tracker<T: PipelineObject + Buildable>(object: T, pipeline: &Pipeline) -> (TrackingTaskID, ObjectID<T>) {
        let object = object.pre_construct(pipeline);
        // Construct it's ID and automatically send it's construction task
        let (t, id) = object.construct_task(pipeline);
        (task_tracker(t, pipeline), id)
    }
    // Detect if a task has executed
    pub fn has_task_executed(id: TrackingTaskID, pipeline: &Pipeline) -> bool {
        pipeline.completed_tasks.contains(&id)
    }
}
