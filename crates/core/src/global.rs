use ecs::{Entity, EntityID, ComponentLinkingGroup};

// Some tasks that we can use whenever we do not have a mutable world
pub(crate) enum WorldTask {
    // ECS
    // Entity Tasks
    AddEntity(Entity, EntityID, ComponentLinkingGroup),
    RemoveEntity(EntityID),
    // Component linking tasks
    DirectAddComponent(EntityID, ComponentLinkingGroup),
}

// This helps us create tasks
pub mod tasks {
    use ecs::{Entity, EntityID, ComponentLinkingGroup};
    use crate::{WorldTask, Context};

    // Create an AddEntity task and send it
    pub fn add_entity(context: &Context, entity: Entity, id: EntityID, group: ComponentLinkingGroup) -> Option<()> {
        context.send(WorldTask::AddEntity(entity, id, group))
    }
    // Create a RemoveEntity task and send it
    pub fn remove_entity(context: &Context, id: EntityID) -> Option<()> {
        context.send(WorldTask::RemoveEntity(id))
    }
}