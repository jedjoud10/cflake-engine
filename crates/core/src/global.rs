use ecs::{Entity, EntityID, ComponentLinkingGroup};
use crate::task::WorldTask;
// This helps us create tasks
pub mod tasks {
    use ecs::{Entity, EntityID, ComponentLinkingGroup};
    use crate::{WorldTask, RefTaskSenderContext};

    // Create an AddEntity task and send it
    pub fn add_entity(sender: &RefTaskSenderContext, entity: Entity, id: EntityID, group: ComponentLinkingGroup) -> Option<()> {
        sender.send(WorldTask::AddEntity(entity, id, group))
    }
    // Create a RemoveEntity task and send it
    pub fn remove_entity(sender: &RefTaskSenderContext, id: EntityID) -> Option<()> {
        sender.send(WorldTask::RemoveEntity(id))
    }
    // Just send a normal task
    pub fn task(sender: &RefTaskSenderContext, task: WorldTask) -> Option<()> {
        sender.send(task)
    }
}