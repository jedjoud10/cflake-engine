use ecs::{EntityID, SystemThreadData};
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
    EntityRemove(EntityID),
    EntityRemovedDecrementCounter(EntityID),
    // Directly add a component linking group to an already existing entity
    AddComponentLinkingGroup(EntityID, ecs::ComponentLinkingGroup),
    // UI
    SetRootVisibility(bool),
}

// Excecute a specific task and give back it's result
pub fn excecute_query(query: CommandQueryType, world: &mut crate::world::World, receiver: &WorldTaskReceiver) {
    use ecs::system::entity_valid;
    // We must extract the normal queries from the batch query if needed
    let queries = match query {
        CommandQueryType::Single(s) => vec![s],
        CommandQueryType::Batch(s) => s,
    };
    // Execute the queries
    for query in queries {
        match query.task {
            Task::EntityAdd(mut entity, linkings) => {
                // Add the entity first
                let entity_cbitfield = linkings.cbitfield;
                entity.cbitfield = linkings.cbitfield;
                let entity_id = world.ecs_manager.add_entity(entity);

                // Nowe add the components
                for (cbitfield, boxed) in linkings.linked_components.into_iter().sorted_by(|(a, _), (b, _)| Ord::cmp(a, b)) {
                    let id = world.ecs_manager.add_component(entity_id, boxed, cbitfield).unwrap();
                }

                // Check the systems where this entity might be valid
                for system in world.ecs_manager.systems() {
                    if entity_valid(&entity_cbitfield, system) {
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
            Task::EntityRemove(id) => {
                // Check if we even have the entity in the first place
                // Run the Entity Remove event on the systems
                let entity = world.ecs_manager.entity(id).unwrap();

                // Check the systems where this entity might be valid
                let valid_systems: Vec<&SystemThreadData> = world
                    .ecs_manager
                    .systems()
                    .iter()
                    .filter(|system| entity_valid(&entity.cbitfield, system))
                    .collect::<Vec<&SystemThreadData>>();
                let count = valid_systems.len() as u8;
                // Send the command to each system
                for system in valid_systems {
                    crate::system::send_lsc(LogicSystemCommand::RemoveEntityFromSystem(id), &system.join_handle.thread().id(), receiver);
                }

                // Only run the callback if we are not on the main thread
                if query.thread_id != std::thread::current().id() {
                    // Tell the main callback manager to execute this callback
                    match query.callback_id {
                        Some(callback_id) => {
                            crate::system::send_lsc(
                                LogicSystemCommand::RunCallback(callback_id, LogicSystemCallbackArguments::EntityRef(id)),
                                &query.thread_id,
                                receiver,
                            );
                        }
                        None => { /* No callback available */ }
                    }
                }

                // Now, we must wait until the next frame to actually delete the entity and it's components
                world.ecs_manager.set_pending_removal_entity(id, count);
            }
            Task::AddComponentLinkingGroup(entity_id, linkings) => {
                // Check if there are any components that are already linked to the entity
                let entity = world.ecs_manager.entity(entity_id).unwrap();
                let new = entity.cbitfield.add(&linkings.cbitfield); // This must be an addition!
                let old = entity.cbitfield;
                if linkings.cbitfield.contains(&old) {
                    // There was a collision!
                    println!(
                        "The components that had a collision are {:?}",
                        ecs::registry::get_component_names_cbitfield(entity.cbitfield)
                    );
                    return;
                }
                // Add the new components onto the entity
                for (cbitfield, boxed) in linkings.linked_components.into_iter().sorted_by(|(a, _), (b, _)| Ord::cmp(a, b)) {
                    let id = world.ecs_manager.add_component(entity_id, boxed, cbitfield).unwrap();
                }

                // Update the entity
                let entity = world.ecs_manager.entity_mut(entity_id).unwrap();
                entity.cbitfield = new;

                // Check the systems so we can add the entity if it became valid for them
                for system in world.ecs_manager.systems() {
                    if entity_valid(&new, system) && !entity_valid(&old, system) {
                        crate::system::send_lsc(LogicSystemCommand::AddEntityToSystem(entity_id), &system.join_handle.thread().id(), receiver);
                    }
                }

                /*
                // Update the components links
                let old_entity_system_bitfield = entity.system_bitfield;
                let combined_c_bitfield = entity.c_bitfield | linkings.c_bitfield;
                // Compare the entity system bitfield
                let new_entity_system_bitfield = calculate_system_bitfield(world, combined_c_bitfield);
                let system_ids_new = new_entity_system_bitfield & !old_entity_system_bitfield; // This is the system bitfield for the systems that did not originally contain this entity because it was invalid, but the entity just became valid for that system
                let entity = world.ecs_manager.entitym.entity_mut(entity_id).unwrap();
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
                */
            }
            Task::SetRootVisibility(_) => {}
            Task::EntityRemovedDecrementCounter(id) => {
                // Decrement the counter, and if we reached 0, we get a Some of the entity
                if let Option::Some(entity) = world.ecs_manager.decrement_removal_counter(id) {
                    let entity = entity.unwrap();
                }
            }
        }
    }
}
