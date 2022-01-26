// Some pipeline commands
pub mod pipec {
    use std::sync::atomic::Ordering;

    use crate::{
        basics::Buildable,
        object::{ObjectID, PipelineObject, PipelineTask, PipelineTaskCombination, PipelineTrackedTask, TrackedTaskID},
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

    // Tracked Tasks
    // Detect if a tracking task has executed
    pub fn has_task_executed(id: TrackedTaskID, pipeline: &Pipeline) -> Option<bool> {
        // This TrackedTaskID might be not finalized, so we must handle that case
        if id.1 {
            Some(pipeline.completed_finalizers.contains(&id))
        } else {
            None
        }
    }
    // Create a tracked task with a requirement
    pub fn tracked_task(task: PipelineTrackedTask, req: Option<TrackedTaskID>, pipeline: &Pipeline) -> TrackedTaskID {
        // Create a new ID for ourselves
        let id = TrackedTaskID::new(false);
        sender::send_task(PipelineTaskCombination::SingleTracked(task, id, req), pipeline).unwrap();
        id
    }
    // Make a finalizer that we can actually use to check if some tasks have all executed
    pub fn tracked_finalizer(reqs: Vec<TrackedTaskID>, pipeline: &Pipeline) -> Option<TrackedTaskID> {
        // Must be the partial version and not the finalized version
        let partial_valid = reqs.iter().all(|x| !x.1);
        if !partial_valid {
            return None;
        }
        let id = TrackedTaskID::new(true);
        sender::send_task(PipelineTaskCombination::SingleTrackedFinalizer(id, reqs), pipeline).unwrap();
        Some(id)
    }
}
