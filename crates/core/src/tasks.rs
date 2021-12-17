use std::collections::HashMap;

use crate::command::CommandQuery;

use crate::global::callbacks::LogicSystemCallbackResultData;
use crate::system::LogicSystemCommand;

// Some world tasks
pub enum Task {
    // Entity
    EntityAdd(ecs::Entity, ecs::ComponentLinkingGroup),
    EntityRemove(usize),
    // This is only valid if the entity is also valid
    ComponentLinkDirect(usize, usize),
    ComponentUnlinkDirect(usize, usize),
    // UI
    SetRootVisibility(bool),
}

// Excecute a specific task and give back it's result
pub fn excecute_query(query: CommandQuery, world: &mut crate::world::World) {
    match query.task {
        Task::EntityAdd(mut entity, linkings) => {
            // Add the components first
            let mut hashmap: HashMap<usize, usize> = HashMap::new();
            for (id, boxed_component) in linkings.linked_components {
                let new_global_id = world.ecs_manager.componentm.add_component(boxed_component).unwrap();
                hashmap.insert(id, new_global_id);
            }
            // Then add the entity
            let entity_id = entity.entity_id;
            entity.linked_components = hashmap;
            world.ecs_manager.entitym.add_entity(entity);

            // Tell the main callback manager to execute this callback
            match query.callback_id {
                Some(id) => {
                    crate::system::send_lsc(LogicSystemCommand::RunCallback(id, LogicSystemCallbackResultData::EntityRef(entity_id)), &query.thread_id);
                }
                None => { /* No callback available */ }
            }
        }
        Task::EntityRemove(_) => {
            // Remove the entity from the world and dispose of it's components
        }
        Task::ComponentLinkDirect(_, _) => {}
        Task::ComponentUnlinkDirect(_, _) => {}
        Task::SetRootVisibility(_) => {}
    }
}
