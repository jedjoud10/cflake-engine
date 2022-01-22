// Some pipeline commands
pub mod pipec {
    use std::sync::atomic::Ordering;

    use crate::{
        basics::Buildable,
        object::{ObjectID, PipelineObject, PipelineTask, TrackingTaskID, PipelineTaskCombination},
        pipeline::{sender, Pipeline},
    };
    // Debug some pipeline data
    pub fn set_debugging(debugging: bool, pipeline: &Pipeline) {
        pipeline.debugging.store(debugging, Ordering::Relaxed);
    }
    // Send a task to the shared pipeline
    pub fn task(task: PipelineTask, pipeline: &Pipeline) {
        sender::send_task(PipelineTaskCombination::Single(task), pipeline).unwrap();
    }
    // Send a batch of tasks all at the same time
    pub fn task_batch(batch: Vec<PipelineTask>, pipeline: &Pipeline) {
        sender::send_task(PipelineTaskCombination::Batch(batch), pipeline).unwrap();
    }
    // Send a task to the shared pipeline, but also return it's tracking ID
    pub fn task_tracker(task: PipelineTask, pipeline: &Pipeline) -> TrackingTaskID {
        // Create a tracking task ID for this task 
        let id = TrackingTaskID::new();
        // Get the thread local sender
        sender::send_task(PipelineTaskCombination::SingleTracked(task, id), pipeline).unwrap();
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
