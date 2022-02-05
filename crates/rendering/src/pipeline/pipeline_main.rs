// Some pipeline commands
pub mod pipec {
    use std::sync::{atomic::Ordering, mpsc::SendError};

    use crate::{
        object::{Deconstruct, DeconstructionTask, ObjectID, PipelineObject, PipelineTask, ReservedTrackedID, TrackedTask, UpdateTask},
        pipeline::{sender, Pipeline, PipelineContext, PipelineRenderer},
    };
    // Debug some pipeline data
    pub fn set_debugging(pipeline: &Pipeline, debugging: bool) {
        pipeline.debugging.store(debugging, Ordering::Relaxed);
    }
    // Send a task to the pipeline
    fn send(pipeline: &Pipeline, task: PipelineTask) -> Option<()> {
        sender::send_task(task, pipeline).ok()
    }
    // Create a Pipeline Object, returning it's ObjectID
    pub fn construct<T: PipelineObject>(pipeline: &Pipeline, object: T) -> Option<ObjectID<T>> {
        // Reseve an ID for the object
        let (object, id) = object.reserve(pipeline)?;

        // Get the PipelineConstructionTask so we can send it to the pipeline
        let task = object.send(pipeline, id);
        send(pipeline, PipelineTask::Construction(task))?;

        // We can now return the object ID
        Some(id)
    }
    // Deconstruct a Pipeline Object, deleting it
    pub fn deconstruct<T: PipelineObject>(pipeline: &Pipeline, id: ObjectID<T>) -> Option<()> {
        id.get()?;
        // Send a deconstruction task to destroy the object
        let task = T::pull(pipeline, id);
        send(pipeline, PipelineTask::Deconstruction(task))?;
        Some(())
    }
    // Update task
    pub fn update_task(pipeline: &Pipeline, task: UpdateTask) -> Option<()> {
        // Send a update task to destroy the object
        send(pipeline, PipelineTask::Update(task))?;
        Some(())
    }
    // Flush the pipeline, forcing the execution of all dispatched tasks
    // This function will exit early and return None if the pipeline is in use, thus we cannot force a flush
    pub fn flush_and_execute(context: &PipelineContext) -> Option<()> {
        // Run the pipeline for one frame, but make sure we have no RwLocks whenever we do so
        let handler = &context.handler.lock().ok()?;
        handler.sbarrier.wait();
        handler.ebarrier.wait();

        // Wait until we wait. Lol
        println!("Waiting for flush completion...");
        while !handler.waiting.load(Ordering::Relaxed) {}
        println!("Flushed!");
        Some(())
    }

    // Tracked Tasks
    // Detect if multiple tasks have all executed
    pub fn did_tasks_execute(pipeline: &Pipeline, ids: &[ReservedTrackedID]) -> bool {
        // Check our sparse bitfield
        let all = ids.iter().all(|x| pipeline.completed_tasks.get(x.0));

        // If they did all execute, we have to reset
        if all {
            ids.iter().for_each(|x| pipeline.completed_tasks.set(x.0, false));
        }
        all
    }
    // Create a tracked task
    pub fn tracked_task(pipeline: &Pipeline, task: TrackedTask, tracked_id: ReservedTrackedID) -> Option<()> {
        send(pipeline, PipelineTask::Tracked(task, tracked_id, None))
    }
    // Create a tracked task with a requirement
    pub fn tracked_task_requirement(pipeline: &Pipeline, task: TrackedTask, tracked_id: ReservedTrackedID, req: ReservedTrackedID) -> Option<()> {
        send(pipeline, PipelineTask::Tracked(task, tracked_id, Some(req)))
    }
    // Add a callback to the pipeline that we will execute at the end of the frame after rendering all the entities
    // This callback will also be called on the render thread, so if we need to do anything with opengl we should use this
    pub fn add_end_of_frame_callback<F: Fn(&mut Pipeline, &mut PipelineRenderer) + Sync + Send + 'static>(pipeline: &Pipeline, function: F) -> Option<()> {
        let mut lock = pipeline.callbacks.lock().ok()?;
        lock.push(Box::new(function));
        Some(())
    }
}
