use std::collections::HashMap;

use crate::communication::*;
use crate::system::{IS_MAIN_THREAD, WORKER_THREADS_RECEIVER};

// Some world tasks
pub enum Task {
    // Entity
    EntityAdd(ecs::Entity, ecs::ComponentLinkingGroup),
    EntityRemove(usize),
    // This is only valid if the entity is also valid
    ComponentLinkDirect(usize, usize),
    ComponentUnlinkDirect(usize, usize),
    // UI
    AddRoot(String, ui::Root),
    SetRootVisibility(bool),
}

// Excecute a specific task and give back it's result
pub fn excecute_task(t: Task, world: &mut crate::world::World) {
    match t {
        Task::EntityAdd(mut entity, linkings) => {
            // Add the components first
            let mut hashmap: HashMap<usize, usize> = HashMap::new();
            for (id, boxed_component) in linkings.linked_components {
                let new_global_id = world.ecs_manager.componentm.add_component(boxed_component).unwrap();
                hashmap.insert(id, new_global_id);
            }
            // Then add the entity
            entity.linked_components = hashmap;
            world.ecs_manager.entitym.add_entity(entity);
        }
        Task::EntityRemove(_) => {
            // Remove the entity from the world and dispose of it's components
        }
        Task::ComponentLinkDirect(_, _) => {}
        Task::ComponentUnlinkDirect(_, _) => {}
        Task::AddRoot(_, _) => {}
        Task::SetRootVisibility(_) => {}
    }
}
