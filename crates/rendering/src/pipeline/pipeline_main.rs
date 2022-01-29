// Some pipeline commands
pub mod pipec {
    use std::sync::{atomic::Ordering, RwLockReadGuard};

    use crate::{
        basics::Buildable,
        object::{ObjectID, PipelineObject, PipelineTask, PipelineTaskCombination, PipelineTrackedTask, ReservedTrackedTaskID},
        pipeline::{sender, Pipeline, PipelineHandler},
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
    // Flush the pipeline, forcing the execution of all dispatched tasks
    pub fn flush_and_execute(pipeline: RwLockReadGuard<Pipeline>, handler: &PipelineHandler) {
        // Run the pipeline for one frame, but make sure we have no RwLocks whenever we do so
        drop(pipeline);
        handler.sbarrier.wait();
        handler.ebarrier.wait();
        
        // Wait until we wait. Lol
        println!("Waiting for flush completion...");
        while !handler.waiting.load(Ordering::Relaxed) {
        }
        println!("Flushed!");
    }


    // Tracked Tasks
    // Detect if a multitude of tasks have all executed
    pub fn did_tasks_execute(ids: &[ReservedTrackedTaskID], pipeline: &Pipeline) -> bool {
        // Check our sparse bitfield
        let all = ids.iter().all(|x| pipeline.completed_tasks.get(x.0));

        // If they did all execute, we have to reset
        if all {
            ids.iter().for_each(|x| pipeline.completed_tasks.set(x.0, false));
        }
        all
    }
    // Create a tracked task
    pub fn tracked_task(task: PipelineTrackedTask, tracked_id: ReservedTrackedTaskID, pipeline: &Pipeline) {
        sender::send_task(PipelineTaskCombination::SingleTracked(task, tracked_id, None), pipeline).unwrap();
    }
    // Create a tracked task with a requirement
    pub fn tracked_task_requirement(task: PipelineTrackedTask, tracked_id: ReservedTrackedTaskID, req: ReservedTrackedTaskID, pipeline: &Pipeline) {
        sender::send_task(PipelineTaskCombination::SingleTracked(task, tracked_id, Some(req)), pipeline).unwrap();
    }
}
