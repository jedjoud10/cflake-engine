use ecs::{Entity, EntityID, ComponentLinkingGroup};
use crate::task::WorldTask;
// This helps us create tasks
pub mod tasks {
    use ecs::{Entity, EntityID, ComponentLinkingGroup};
    use crate::{WorldTask, TaskSenderContext};

    // Create an AddEntity task and send it
    pub fn add_entity(sender: &TaskSenderContext, entity: Entity, id: EntityID, group: ComponentLinkingGroup) -> Option<()> {
        sender.send(WorldTask::AddEntity(entity, id, group))
    }
    // Create a RemoveEntity task and send it
    pub fn remove_entity(sender: &TaskSenderContext, id: EntityID) -> Option<()> {
        sender.send(WorldTask::RemoveEntity(id))
    }
    // Just send a normal task
    pub fn task(sender: &TaskSenderContext, task: WorldTask) -> Option<()> {
        sender.send(task)
    }
}