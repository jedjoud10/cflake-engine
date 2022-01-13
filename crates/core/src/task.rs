use ecs::{EntityID, Entity, ComponentLinkingGroup};

use crate::Context;

// A task sender context that we can use to send tasks
pub struct TaskSenderContext {
    // The task timing for this context
    timing: WorldTaskTiming,
}

impl TaskSenderContext {
    // New
    pub fn new(context: &Context) -> Self {
        Self {
            timing: WorldTaskTiming::default(),
        }
    }
    // With timing
    pub fn set_timing(mut self, timing: WorldTaskTiming) -> Self {
        self.timing = timing;
        self
    }
    // Send a task to the main thread
    pub(crate) fn send(self, task: WorldTask) {
        crate::sender::send_task(task)
    }
    // Send a batch of tasks to the main thread
    pub(crate) fn send_batch(self, tasks: Vec<WorldTask>) {
        todo!()
    }
}

// Some tasks that we can use whenever we do not have a mutable world
pub enum WorldTask {
    // ECS
    // Entity Tasks
    AddEntity(Entity, EntityID, ComponentLinkingGroup),
    RemoveEntity(EntityID),
    // Component linking tasks
    DirectAddComponent(EntityID, ComponentLinkingGroup),
}

// Some additional data telling the main thread when it should execute the task
pub enum WorldTaskTiming {
    // We should excute the task as soon as possible, so before the next system executes
    Earliest,

    // We should execute the task at the the end of the frame, so it would be completed for next frame
    ByNextFrame,

    /*
    // We should execute the task so it would be completed by the 'N'th frame
    ByFrame(u64),
    */

    // Doesn't matter whenever we execute the task, so we should execute it only when we have some spare time 
    Free,
}

impl Default for WorldTaskTiming { 
    fn default() -> Self {
        Self::Earliest
    } 
}