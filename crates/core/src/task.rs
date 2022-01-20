use crate::Context;
use ecs::entity::*;

// A task sender context that we can use to send tasks to the main thread
pub struct TaskSenderContext {
}

impl TaskSenderContext {
    // New
    pub fn new(_context: &Context) -> Self {
        Self {}
    }
    // Send a task to the main thread
    pub(crate) fn send(&self, task: WorldTask) -> Option<()> {
        crate::sender::send_task(WorldTaskBatch {
            combination: WorldTaskCombination::Single(task),
        })
    }
    // Send a batch of tasks to the main thread
    pub(crate) fn send_batch(&self, tasks: Vec<WorldTask>) -> Option<()> {
        crate::sender::send_task(WorldTaskBatch {
            combination: WorldTaskCombination::Batch(tasks),
        })
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

pub(crate) enum WorldTaskCombination {
    Batch(Vec<WorldTask>),
    Single(WorldTask),
}

// A batch of tasks
pub struct WorldTaskBatch {
    pub(crate) combination: WorldTaskCombination,
}
