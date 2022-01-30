use main::{
    core::{Context, WriteContext},
    ecs::{self, component::ComponentQuery, entity::EntityID},
    terrain::{ChunkCoords},
};

// Add a single chunk to the world
fn add_chunk(write: &mut WriteContext, octree_size: u64, coords: ChunkCoords) -> EntityID {
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

    // Chunk
    let chunk = crate::components::Chunk::new(coords);
    group.link::<crate::components::Chunk>(chunk).unwrap();

    // Add the entity to the world
    write.ecs.add_entity(entity, id, group).unwrap();
    println!("Spawn chunk at {} with EntityID: {}", coords.center, id);
    id
}
// Remove a single chunk
fn remove_chunk(write: &mut WriteContext, id: EntityID) {
    // Make sure that the chunk entity even exists
    if write.ecs.entity(&id).is_ok() {
        // Remove the chunk entity at that specific EntityID
        write.ecs.remove_entity(id).unwrap();
        println!("Remove chunk with EntityID: {}", id);
    }
}

// The chunk systems' update loop
fn run(context: &mut Context, _query: ComponentQuery) {
    // Get the global terrain component
    let mut write = context.write();
    // Get the camera position
    let camera_pos = write.ecs.global::<crate::globals::GlobalWorldData>().unwrap().camera_pos;
    let terrain = write.ecs.global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // Generate the chunks if needed and only if we are not currently generating 
        if !terrain.generating {
            let octree = &mut terrain.octree;
            if let Some((added, removed)) = octree.update(camera_pos) {
                terrain.swap_chunks = false;
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
                        let id = add_chunk(&mut write, terrain.octree.inner.size, coords);
                        terrain.chunks.insert(coords, id);
                    }
                }
                return;
            }

            // Mass deletion
            if terrain.swap_chunks {
                let chunks_to_remove = std::mem::take(&mut terrain.chunks_to_remove);
                for id in chunks_to_remove {
                    remove_chunk(&mut write, id)
                }
            }
        }
    }
}
// Create a chunk system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().set_run_event(run).build()
}
