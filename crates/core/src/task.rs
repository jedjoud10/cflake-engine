use ecs::entity::*;

// A task sender context that we can use to send tasks to the main thread
// We can only create this using the ShareableContext
pub struct TaskSenderContext(pub(crate) ());

impl TaskSenderContext {
    // Send a task to the main thread
    pub(crate) fn send(&self, task: WorldTask) -> Option<()> {
        crate::sender::send_task(task)
    }
}

// Some tasks that we can use whenever we do not have a mutable world
pub enum WorldTask {
    // ECS
    // Entity Tasks
    AddEntity(Entity, EntityID, ComponentLinkingGroup),
    RemoveEntity(EntityID),
    // Component linking tasks
    DirectLinkComponents(EntityID, ComponentLinkingGroup),
    DirectRemoveComponents(EntityID, ComponentUnlinkGroup),
}