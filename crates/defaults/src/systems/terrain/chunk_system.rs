use std::cmp::Ordering;

use main::{
    core::{Context, WriteContext},
    ecs::{self, component::ComponentQuery, entity::EntityID},
    input::Keys,
    terrain::ChunkCoords,
};

// Add a single chunk to the world
fn add_chunk(write: &mut WriteContext, camera_position: veclib::Vector3<f32>, camera_direction: veclib::Vector3<f32>, octree_size: u64, coords: ChunkCoords) -> (EntityID, f32) {
    // Create the chunk entity
    let entity = ecs::entity::Entity::default();
    let id = ecs::entity::EntityID::new(&write.ecs);
    let mut group = ecs::entity::ComponentLinkingGroup::default();

    // Link the nessecary components
    // Transform
    let position = veclib::Vector3::<f32>::from(coords.position);
    let scale = veclib::Vector3::ONE * ((coords.size / octree_size) as f32);
    let transform = crate::components::Transform::default().with_position(position).with_scale(scale);
    group.link::<crate::components::Transform>(transform).unwrap();

    // Calculate the chunk's priory and create it
    let priority = (camera_direction - position).dot(camera_direction);
    let chunk = crate::components::Chunk::new(coords);
    group.link::<crate::components::Chunk>(chunk).unwrap();

    // Add the entity to the world
    write.ecs.add_entity(entity, id, group).unwrap();
    (id, priority)
}
// Remove a single chunk
fn remove_chunk(write: &mut WriteContext, id: EntityID) {
    // Make sure that the chunk entity even exists
    if write.ecs.get_entity(&id).is_ok() {
        // Remove the chunk entity at that specific EntityID
        write.ecs.remove_entity(id).unwrap();
    }
}

// The chunk systems' update loop
fn run(context: &mut Context, _query: ComponentQuery) {
    // Get the global terrain component
    let mut write = context.write();
    // Get the camera position
    let (camera_pos, camera_dir) = {
        let cam = write.ecs.get_global::<crate::globals::GlobalWorldData>().unwrap();
        (cam.camera_pos, cam.camera_dir)
    };
    let terrain = write.ecs.get_global_mut::<crate::globals::Terrain>();
    if write.input.map_toggled("update_terrain") { return; }
    if let Ok(mut terrain) = terrain {
        // Generate the chunks if needed and only if we are not currently generating
        if terrain.chunks_generating.is_empty() && terrain.chunks_to_remove.is_empty() {
            let octree = &mut terrain.octree;
            if let Some((added, removed)) = octree.update(camera_pos) {
                // We have moved, thus the chunks need to be regenerated
                // Remove chunks only if we already generated them
                for node in removed {
                    let coords = ChunkCoords::new(&node);
                    if let Some(id) = terrain.chunks.remove(&coords) {
                        terrain.chunks_to_remove.push(id);
                    }
                }

                // Only add the chunks that are leaf nodes in the octree
                for node in added {
                    if node.children_indices.is_none() {
                        // This is a leaf node
                        let coords = ChunkCoords::new(&node);
                        let (id, priority) = add_chunk(&mut write, camera_pos, camera_dir, terrain.octree.inner.size, coords);
                        terrain.sorted_chunks_generating.push((id, priority));
                        terrain.chunks.insert(coords, id);
                        terrain.chunks_generating.insert(coords);
                    }
                }
                terrain.sorted_chunks_generating.sort_by(|(_, x), (_, y)| f32::partial_cmp(x, y).unwrap_or(Ordering::Equal));
                return;
            }
        } else {
            // Mass deletion when we have no more chunks
            if terrain.chunks_generating.is_empty() {
                let chunks_to_remove = std::mem::take(&mut terrain.chunks_to_remove);
                for id in chunks_to_remove {
                    remove_chunk(&mut write, id);
                }
            }
        }
    }
}
// Create a chunk system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).build();
    write.input.bind_key_toggle(Keys::Y, "update_terrain");
}
