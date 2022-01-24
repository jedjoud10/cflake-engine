use main::{
    core::{Context, WriteContext},
    ecs::{component::ComponentQuery, self, entity::EntityID},
    terrain::{DEFAULT_LOD_FACTOR, ChunkCoords}, math::octrees::OctreeNode,
};

// Add a single chunk to the world
fn add_chunk(write: &mut WriteContext, octree_size: u64, coords: ChunkCoords) -> EntityID {
    // Create the chunk entity
    let entity = ecs::entity::Entity::new();
    let id = ecs::entity::EntityID::new(&write.ecs);
    let mut group = ecs::entity::ComponentLinkingGroup::new();

    // Link the nessecary components
    // Transform
    let position = veclib::Vector3::<f32>::from(coords.position);
    let scale = veclib::Vector3::ONE * ((coords.size / octree_size) as f32);
    let transform = crate::components::Transform::default()
        .with_position(position)
        .with_scale(scale);
    group.link::<crate::components::Transform>(transform).unwrap();

    // Chunk
    let chunk = crate::components::Chunk::new(coords);
    group.link::<crate::components::Chunk>(chunk).unwrap();

    // Add the entity to the world
    write.ecs.add_entity(entity, id, group).unwrap();
    id
}
// Remove a single chunk
fn remove_chunk(write: &mut WriteContext, id: EntityID) {
    // Remove the chunk entity at that specific EntityID
    write.ecs.remove_entity(id).unwrap();
}

// The chunk systems' update loop
fn run(mut context: Context, _query: ComponentQuery) {
    // Get the global terrain component
    let mut write = context.write();
    // Get the camera position
    let camera_pos = write.ecs.global::<crate::globals::GlobalWorldData>().unwrap().camera_pos;
    let terrain = write.ecs.global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // Generate the chunks if needed
        let octree = &mut terrain.octree;
        if let Some((added, removed)) = octree.generate_incremental_octree(&camera_pos, DEFAULT_LOD_FACTOR) {
            // We have moved, thus the chunks need to be regenerated
            
            // Only add the chunks that are leaf nodes in the octree
            for node in added {
                if node.children_indices.is_none() {
                    // This is a leaf node
                    let coords = ChunkCoords::new(&node);
                    let id = add_chunk(&mut write, terrain.octree.internal_octree.size, coords);
                    terrain.chunks.insert(coords, id);
                }
            } 

            // Remove chunks only if we already generated them
            for node in removed {
                let coords = ChunkCoords::new(&node);
                if let Some(&id) = terrain.chunks.get(&coords) {
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
