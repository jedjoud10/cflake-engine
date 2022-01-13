use ecs::{Entity, EntityID, ComponentLinkingGroup};
use crate::task::WorldTask;
// This helps us create tasks
pub mod tasks {
    use ecs::{Entity, EntityID, ComponentLinkingGroup};
    use crate::{WorldTask, Context, TaskSenderContext};

    // Create an AddEntity task and send it
    pub fn add_entity(task_sender_context: &TaskSenderContext, entity: Entity, id: EntityID, group: ComponentLinkingGroup) -> Option<()> {
        context.send(WorldTask::AddEntity(entity, id, group))
    }
    // Create a RemoveEntity task and send it
    pub fn remove_entity(task_sender_context: &TaskSenderContext, id: EntityID) -> Option<()> {
        context.send(WorldTask::RemoveEntity(id))
    }
    // Just send a normal task
    pub fn task(task_sender_context: &TaskSenderContext, task: WorldTask) -> Option<()> {
        context.send(task)
    }
}