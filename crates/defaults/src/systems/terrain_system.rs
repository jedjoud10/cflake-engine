use others::Instance;
use rendering::pipec;
use std::{rc::Rc, time::Instant};
use terrain::{ChunkCoords, TerrainStats};

use crate::components;
use components::Chunk;
use ecs::*;
use math::octrees::*;
use systems::*;
use world_data::WorldData;

fn entity_update(system_data: &mut SystemData, _entity: &Entity, components: &FilteredLinkedComponents, data: &mut WorldData) {
    // Timing
    let t = Instant::now();
    // Get the camera location
    let camera_entity = data.entity_manager.get_entity(data.custom_data.main_camera_entity_id).unwrap();
    let camera_transform = camera_entity.get_component::<components::Transform>(data.component_manager).unwrap();
    // Get the camera transform values
    let camera_position = camera_transform.position;
    let camera_forward_vector = camera_transform.get_forward_vector();

    // Get the terrain data
    let td = components.get_component_mut::<components::TerrainData>(data.component_manager).unwrap();
    let terrain = &mut td.terrain;
    let material = terrain.settings.material.clone();
    let octree_size = terrain.octree.internal_octree.size;
    let octree_depth = terrain.settings.octree_depth;

    // Generate the octree each frame and generate / delete the chunks
    if terrain.chunk_manager.octree_update_valid() && !data.input_manager.map_toggled("toggle-terrain-gen") {
        match terrain.octree.generate_incremental_octree(&camera_position, terrain::DEFAULT_LOD_FACTOR) {
            Some((mut added, removed, total)) => {
                // Filter first
                added.retain(|node| node.children_indices.is_none() && math::Intersection::csgtree_aabb(&terrain.csgtree, &node.get_aabb()));
                // Turn all the newly added nodes into chunks and instantiate them into the world
                for octree_node in added {
                    // Add the chunk in the chunk manager
                    terrain.chunk_manager.add_chunk(ChunkCoords::new(&octree_node));
                }
                // Delete all the removed octree nodes from the world
                for octree_node in removed {
                    let chunk_coords = ChunkCoords::new(&octree_node);
                    // Remove the chunk from the chunk manager
                    match terrain.chunk_manager.remove_chunk(&chunk_coords) {
                        Some(_) => {
                            // Get the entity id
                            terrain.chunk_manager.remove_chunk_entity(&chunk_coords);
                        }
                        None => {}
                    }
                }
            }
            None => { /* Nothing happened */ }
        }
        terrain.chunk_manager.update_camera_view(camera_position, camera_forward_vector);
    }

    // Update the chunk manager
    match terrain.chunk_manager.update(&mut terrain.voxel_generator, data.time_manager.frame_count) {
        Some((added_chunks, removed_chunks)) => {
            let mut added_chunk_entities_ids: Vec<(usize, ChunkCoords)> = Vec::new();
            let i = std::time::Instant::now();
            // Add the entities to the entity manager
            for (coords, tmodel) in added_chunks {
                // Add the entity
                let name = format!("Chunk {:?} {:?}", coords.position, coords.size);
                let mut entity = Entity::new(name.as_str());

                // Create the chunk component
                let chunk = Chunk { coords: coords.clone() };
                // Link the components
                entity.link_component::<Chunk>(data.component_manager, chunk).unwrap();
                // Transform
                entity
                    .link_component::<components::Transform>(
                        data.component_manager,
                        components::Transform {
                            position: veclib::Vector3::<f32>::from(coords.position),
                            scale: veclib::Vector3::new((coords.size / octree_size) as f32, (coords.size / octree_size) as f32, (coords.size / octree_size) as f32),
                            ..components::Transform::default()
                        },
                    )
                    .unwrap();

                // Turn the model into a GPU model
                let model = pipec::model(tmodel.model);
                let renderer = components::Renderer::default().set_material(material.clone()).set_wireframe(true).set_model(model);
                entity.link_component::<components::Renderer>(data.component_manager, renderer).unwrap();
                // Create the AABB
                let aabb = components::AABB::from_components(&entity, data.component_manager);
                //entity.link_component::<components::AABB>(data.component_manager, aabb).unwrap();
                let entity_id = data.entity_manager.add_entity_s(entity);
                added_chunk_entities_ids.push((entity_id, coords.clone()));
            }
            let x = i.elapsed().as_millis();
            if x != 0 {
                println!("Elapsed: {}", x);
            }
            // Reassign
            let td = components.get_component_mut::<components::TerrainData>(data.component_manager).unwrap();
            let terrain = &mut td.terrain;
            for (entity_id, coords) in added_chunk_entities_ids {
                terrain.chunk_manager.add_chunk_entity(&coords, entity_id);
            }

            for entity_id in removed_chunks {
                // Removal the entity from the world
                data.entity_manager.remove_entity_s(entity_id).unwrap();
            }
        }
        None => {}
    }
}
fn entity_added(_system_data: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    // Setup the voxel generator for this generator
    let td = entity.get_component_mut::<components::TerrainData>(data.component_manager).unwrap();
    // Generate the voxel texture
    td.terrain.voxel_generator.setup_voxel_generator();
}

// Create the terrain system
pub fn system(data: &mut WorldData) -> System {
    let mut system = System::default();
    // Link the components
    system.link_component::<components::TerrainData>(data.component_manager).unwrap();
    data.component_manager.register_component::<Chunk>();

    data.input_manager.bind_key(input::Keys::B, "toggle-terrain-gen", input::MapType::Toggle);
    // Attach the events
    system.event(SystemEventType::EntityAdded(entity_added));
    system.event(SystemEventType::EntityUpdate(entity_update));
    system
}
