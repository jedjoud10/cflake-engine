use std::collections::HashMap;
use itertools::Itertools;

use crate::command::CommandQuery;
use crate::communication::WorldTaskReceiver;
use crate::global::callbacks::LogicSystemCallbackArguments;
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

// This is some immediate result that is given after we execute a query command on the main thread
pub enum ImmediateTaskResult {
    None,
    EntityAdd(usize),
}

impl ImmediateTaskResult {
    // Get the Entity ID in case we have executed something that returns an entity ID
    pub fn entity_id(self) -> Option<usize> {
        match self {
            ImmediateTaskResult::None => None,
            ImmediateTaskResult::EntityAdd(id) => Some(id),
        }
    }
}

// Excecute a specific task and give back it's result
pub fn excecute_query(query: CommandQuery, world: &mut crate::world::World, receiver: &WorldTaskReceiver) -> ImmediateTaskResult {
    match query.task {
        Task::EntityAdd(mut entity, mut linkings) => {
            // Add the components first
            let mut hashmap: HashMap<usize, usize> = HashMap::new();
            for (id, boxed_component) in linkings.linked_components.into_iter().sorted_by(|(a, _), (b, _)| Ord::cmp(a, b)) {
                let new_global_id = world.ecs_manager.componentm.add_component(boxed_component).unwrap();
                hashmap.insert(id, new_global_id);
            }
            // Set the entity values
            let entity_id = world.ecs_manager.entitym.entities.get_next_valid_id();
            let entity_cbitfield = linkings.c_bitfield;

            entity.entity_id = entity_id;
            entity.linked_components = hashmap;
            entity.c_bitfield = entity_cbitfield;
            dbg!(&entity);
            // Then add the entity
            world.ecs_manager.entitym.add_entity(entity);

            // Check if a specified entity fits the criteria to be in a specific system
            fn is_entity_valid(system_c_bitfield: usize, entity_c_bitfield: usize) -> bool {
                // Check if the system matches the component ID of the entity
                let bitfield: usize = system_c_bitfield & !entity_c_bitfield;
                // If the entity is valid, all the bits would be 0
                bitfield == 0
            }

            // Check the systems where this entity might be valid
            for system in world.ecs_manager.systemm.systems.iter() {
                let valid = is_entity_valid(system.c_bitfield, entity_cbitfield);
                if valid {
                    crate::system::send_lsc(LogicSystemCommand::AddEntityToSystem(entity_id), &system.join_handle.thread().id(), receiver);
                }
            }

            // Only run the callback if we are not on the main thread
            if query.thread_id != std::thread::current().id() {
                // Tell the main callback manager to execute this callback
                match query.callback_id {
                    Some(id) => {
                        crate::system::send_lsc(
                            LogicSystemCommand::RunCallback(id, LogicSystemCallbackArguments::EntityRef(entity_id)),
                            &query.thread_id,
                            receiver,
                        );
                    }
                    None => { /* No callback available */ }
                }
            }
            ImmediateTaskResult::EntityAdd(entity_id)
        }
        Task::EntityRemove(_) => {
            // Remove the entity from the world and dispose of it's components
            // When doing this however, we must wait a whole frame before actually deleting the entity
            // We must first send the LogicSystemCommand to each system that contains this entity, then we can actually delete the entity the next frame
            ImmediateTaskResult::None
        }
        Task::ComponentLinkDirect(_, _) => ImmediateTaskResult::None,
        Task::ComponentUnlinkDirect(_, _) => ImmediateTaskResult::None,
        Task::SetRootVisibility(_) => ImmediateTaskResult::None,
    }
}
