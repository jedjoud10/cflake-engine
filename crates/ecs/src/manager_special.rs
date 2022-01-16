use crate::{ECSManager, entity::{EntityID, Entity, ComponentLinkingGroup, ComponentUnlinkGroup}, utils::{EntityError, ComponentError}};

// Add an entity to the manager, and automatically link it's components
pub fn add_entity<Context>(ecs: &mut ECSManager<Context>, mut entity: Entity, id: EntityID, group: ComponentLinkingGroup) {
    ecs.add_entity(entity, id, group)
}
// Remove an entity from the manager, and return it's value
// When we remove an entity, we also remove it's components, thus updating the systems
pub fn remove_entity<Context>(ecs: &mut ECSManager<Context>, id: EntityID) -> Result<Entity, EntityError> {
    ecs.remove_entity(id)
}
// Link some components to an entity
pub fn link_components<Context>(ecs: &mut ECSManager<Context>, id: EntityID, link_group: ComponentLinkingGroup) -> Result<(), ComponentError> {
    ecs.link_components(id, link_group)
}
// Unlink some components from an entity
pub fn unlink_components<Context>(ecs: &mut ECSManager<Context>, id: EntityID, unlink_group: ComponentUnlinkGroup) -> Result<(), ComponentError> {
    ecs.unlink_components(id, unlink_group)
}