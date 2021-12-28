use ecs::SystemThreadData;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;

use crate::command::{CommandQuery, CommandQueryType};
use crate::communication::WorldTaskReceiver;
use crate::global::callbacks::LogicSystemCallbackArguments;
use crate::system::LogicSystemCommand;

// Some world tasks
pub enum Task {
    // Entity
    EntityAdd(ecs::Entity, ecs::ComponentLinkingGroup),
    EntityRemove(usize),
    EntityRemovedDecrementCounter(usize),
    // Directly add a component linking group to an already existing entity
    AddComponentLinkingGroup(usize, ecs::ComponentLinkingGroup),
    // UI
    SetRootVisibility(bool),
}

// Excecute a specific task and give back it's result
pub fn excecute_query(query: CommandQueryType, world: &mut crate::world::World, receiver: &WorldTaskReceiver) {
    // We must extract the normal queries from the batch query if needed
    let queries = match query {
        CommandQueryType::Single(s) => vec![s],
        CommandQueryType::Batch(s) => s,
    };
    // Check if a specified entity fits the criteria to be in a specific system
    fn is_entity_valid(system_c_bitfield: usize, entity_c_bitfield: usize) -> bool {
        // Component Bitfield Test
        // entity:  100001 -> 011110
        // system1: 001001 -> 001001 -> 001000 -> INVALID
        // system2: 100000 -> 100010 -> 000010 -> VALID
        // Check if the system matches the component ID of the entity
        let bitfield: usize = system_c_bitfield & !entity_c_bitfield;
        // If the entity is valid, all the bits would be 0
        let entity_valid = bitfield == 0;
        // If the systems has no components to it, we must not link the entity
        let system_valid = system_c_bitfield != 0;
        entity_valid && system_valid
    }
    // Calculate the system bitfield from an entity component bitfield
    fn calculate_system_bitfield(world: &crate::world::World, entity_c_bitfield: usize) -> u32 {
        let mut system_bitfield = 0;
        for system in world.ecs_manager.systemm.systems.iter() {
            let valid = is_entity_valid(system.c_bitfield, entity_c_bitfield);
            if valid {
                system_bitfield |= system.system_id;
            }
        }
        system_bitfield
    }
    // Execute the queries
    for query in queries {
        match query.task {
            Task::EntityAdd(mut entity, linkings) => {
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

                // Calculate the system bitfield
                entity.system_bitfield = calculate_system_bitfield(world, entity.c_bitfield);

                // Then add the entity
                world.ecs_manager.entitym.add_entity(entity);

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
            }
            Task::EntityRemove(entity_id) => {
                // Check if we even have the entity in the first place
                if !world.ecs_manager.entitym.is_entity_valid(entity_id) {
                    continue;
                }
                // Run the Entity Remove event on the systems
                let entity = world.ecs_manager.entitym.entity(entity_id);

                // Check the systems where this entity might be valid
                let valid_systems: Vec<&SystemThreadData> = world
                    .ecs_manager
                    .systemm
                    .systems
                    .iter()
                    .filter(|system| is_entity_valid(system.c_bitfield, entity.c_bitfield))
                    .collect::<Vec<&SystemThreadData>>();
                let count = valid_systems.len() as u8;
                // Send the command to each system
                for system in valid_systems {
                    crate::system::send_lsc(LogicSystemCommand::RemoveEntityFromSystem(entity_id), &system.join_handle.thread().id(), receiver);
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

                // Now, we must wait until the next frame to actually delete the entity and it's components
                world.ecs_manager.entitym.entities_to_delete.insert(entity_id, count);
            }
            Task::AddComponentLinkingGroup(entity_id, linkings) => {
                // Check if there are any components that are already linked to the entity
                let entity = world.ecs_manager.entitym.entity(entity_id);
                let collision_cbitfield = entity.c_bitfield & linkings.c_bitfield;
                if collision_cbitfield != 0 {
                    // There was a collision!
                    panic!(
                        "The components that had a collision are {:?}",
                        ecs::registry::get_component_names_cbitfield(collision_cbitfield)
                    );
                }
                // Add the new components onto the entity
                let mut hashmap: HashMap<usize, usize> = HashMap::new();
                for (id, boxed_component) in linkings.linked_components.into_iter().sorted_by(|(a, _), (b, _)| Ord::cmp(a, b)) {
                    let new_global_id = world.ecs_manager.componentm.add_component(boxed_component).unwrap();
                    hashmap.insert(id, new_global_id);
                }
                // Update the components links
                let old_entity_system_bitfield = entity.system_bitfield;
                let combined_c_bitfield = entity.c_bitfield | linkings.c_bitfield;
                // Compare the entity system bitfield
                let new_entity_system_bitfield = calculate_system_bitfield(world, combined_c_bitfield);
                let system_ids_new = new_entity_system_bitfield & !old_entity_system_bitfield; // This is the system bitfield for the systems that did not originally contain this entity because it was invalid, but the entity just became valid for that system
                let entity = world.ecs_manager.entitym.entity_mut(entity_id);
                entity.linked_components.extend(hashmap);
                entity.c_bitfield = combined_c_bitfield;
                entity.system_bitfield = new_entity_system_bitfield;
                // Signal the systems who have this entity as a new entity
                for system in world.ecs_manager.systemm.systems.iter() {
                    // System Bitfield Test
                    // entity:  100101 -> 011010
                    // system1: 000001 -> 000001 -> 000000 -> VALID
                    // system2: 000010 -> 000010 -> 000010 -> INVALID

                    // Check if this system can actually add the entity internally
                    if (system.system_id & !system_ids_new) == 0 {
                        crate::system::send_lsc(LogicSystemCommand::AddEntityToSystem(entity_id), &system.join_handle.thread().id(), receiver);
                    }
                }
            }
            Task::SetRootVisibility(_) => {}
            Task::EntityRemovedDecrementCounter(entity_id) => {
                let counter = {
                    // One of the systems has safely removed the entity from it's list, so we must decrement the counter
                    let counter = world.ecs_manager.entitym.entities_to_delete.get_mut(&entity_id).unwrap();
                    *counter -= 1;
                    *counter
                };
                // If the counter has reached 0, we can safely remove the entity
                if counter == 0 {
                    // Delete the entity and it's corresponding components
                    world.ecs_manager.entitym.entities_to_delete.remove(&entity_id);
                    let entity = world.ecs_manager.entitym.remove_entity(entity_id);
                    for (component_id, global_id) in entity.linked_components {
                        world.ecs_manager.componentm.remove_component(global_id).unwrap();
                    }
                }
            }
        }
    }
}
