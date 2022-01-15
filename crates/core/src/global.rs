// This helps us create tasks
pub mod tasks {
    use crate::{TaskSenderContext, WorldTask};

    // Tasks related only to ECS
    pub mod ecs {
        use crate::{TaskSenderContext, WorldTask};
        use ecs::entity::*;

        // Create an AddEntity task and send it
        pub fn add_entity(sender: &TaskSenderContext, entity: Entity, id: EntityID, group: ComponentLinkingGroup) -> Option<()> {
            sender.send(WorldTask::AddEntity(entity, id, group))
        }
        // Create a RemoveEntity task and send it
        pub fn remove_entity(sender: &TaskSenderContext, id: EntityID) -> Option<()> {
            sender.send(WorldTask::RemoveEntity(id))
        }
    }
    // Just send a normal task
    pub fn task(sender: &TaskSenderContext, task: WorldTask) -> Option<()> {
        sender.send(task)
    }
}
