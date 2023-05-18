use crate::{
    Chunk, ChunkManager, ChunkState, ChunkViewer, Terrain, TerrainMaterial,
    TerrainSettings,
};
use ahash::{AHashMap, AHashSet};

use coords::{Position, Rotation, Scale};
use ecs::{Entity, Scene};

use graphics::{
    ActivePipeline, ComputePass, DrawIndexedIndirect, DrawIndexedIndirectBuffer, Graphics,
};
use math::OctreeDelta;
use rand::{Rng, seq::SliceRandom};
use rendering::{IndirectMesh, Renderer, Surface, MultiDrawIndirectMesh, MultiDrawIndirectCountMesh};
use utils::{Storage, Time};
use world::{user, System, World};

// Dynamically generate the chunks based on camera position
fn update(world: &mut World) {
    // Tries to find a chunk viewer and the terrain generator
    let terrain = world.get_mut::<Terrain>();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let time = world.get::<Time>().unwrap();
    let viewer = scene.find_mut::<(&Entity, &mut ChunkViewer, &Position, &Rotation)>();

    // If we don't have terrain, don't do shit
    let Ok(mut _terrain) = terrain else {
        return;
    };

    if !_terrain.active {
        return;
    }

    // Get the terrain chunk manager and terrain settings
    let terrain = &mut *_terrain;
    let mut manager = &mut terrain.manager;
    let memory = &mut terrain.memory;
    let settings = &terrain.settings;

    // If we don't have a chunk viewer, don't do shit
    let Some((entity, _, viewer_position, viewer_rotation)) = viewer else {
        manager.viewer = None;
        return;
    };

    // Fetch the old chunk viewer position value
    let mut added = false;
    let new = **viewer_position;
    let old = if let Some((_, old, _)) = &mut manager.viewer {
        *old
    } else {
        manager.viewer = Some((*entity, new, **viewer_rotation));
        added = true;
        new
    };

    // Makes sure that we don't generate when we're not done
    // The only incovenience we get with this approach is that terrain takes a bit more time to update when we're moving very fast
    // Skil issue tbh
    let count = scene.query_mut::<&mut Chunk>().into_iter()
        .filter(|c| c.state == ChunkState::Pending || c.state == ChunkState::Dirty || c.state == ChunkState::PendingReadbackStart || c.state == ChunkState::PendingReadbackData || c.state == ChunkState::Removed)
        .count();

    // Check if it moved since last frame
    if (added || new != old) && count == 0 {
        let befores = manager.octree.nodes().into_iter().map(|x| (x.center(), x)).collect::<AHashMap<_, _>>();
        let before_ambatakum = manager.octree.nodes().to_vec();

        // Update the old chunk viewer position value
        if let Some((_, val, _)) = manager.viewer.as_mut() {
            *val = new;
        }

        // Regenerate the octree and detect diffs
        let OctreeDelta {
            mut added,
            removed
        } = manager.octree.compute(new);

        // If a children node was generated, then the parent node must've also been generated
        manager.counting.clear();
        for parent in added.iter().filter(|x| x.children().is_some()) {
            let old = manager.counting.insert(parent.center(), (0, Vec::new()));
            assert!(old.is_none());
        }

        for x in added.iter() { 
            let old = manager.parent_node_children_generated.insert(x.center(), false);
            //assert!(old.is_none())
        }

        // Discard non-leaf nodes
        added.retain(|x| x.leaf());

        // Don't do shit
        if added.is_empty() && removed.is_empty() {
            return;
        }

        // If we don't add chunks just exit
        if added.is_empty() {
            return;
        }

        // Set the chunk state to "removed" so we can hide it when it's children all generated
        for node in removed {
            if let Some(entity) = manager.entities.remove(&node) {
                let mut entry = scene.entry_mut(entity).unwrap();
                let chunk = entry.get_mut::<Chunk>().unwrap();
                chunk.state = ChunkState::Removed;
                assert_eq!(chunk.node, Some(node));
            }
            
            manager.parent_node_children_generated.remove(&node.center());
        }

        // Get the number of free chunks that we can reuse
        let query_count = scene
            .query_mut::<&Chunk>()
            .into_iter()
            .filter(|x| x.state == ChunkState::Free)
            .count();

        // Add extra chunks if we need them
        let mut rng = rand::thread_rng();
        let mut multi_draw_indirect_count_meshes = world.get_mut::<Storage<MultiDrawIndirectCountMesh>>().unwrap();
        let mut indexed_indirect_buffers = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();

        // Extend the indexed indirect buffer if needed
        if added.len() > query_count {
            // Over-allocate so we don't need to do this as many times
            let count = ((added.len() - query_count) * 2).max(128);

            // Keep track of the entities we will add
            let mut entities: Vec<(Position, Scale, Chunk)> = Vec::new(); 

            // Keep track of the old number of chunks
            let old = manager.chunks_per_allocation;
            let old_per_allocation = old / settings.allocation_count;

            // Add the same amounts of chunks per allocation
            let mut global_index = old;
            for allocation in 0..settings.allocation_count { 
                // Extend the generated indirect draw buffer
                memory.generated_indexed_indirect_buffers[allocation].extend_from_slice(&vec![
                    crate::util::DEFAULT_DRAW_INDEXED_INDIRECT;
                    count
                ]).unwrap();

                // Extend the culled indirect draw buffer
                let handle = &memory.culled_indexed_indirect_buffers[allocation];
                let culled_indexed_indirect_buffer = indexed_indirect_buffers.get_mut(handle);
                culled_indexed_indirect_buffer.extend_from_slice(&vec![
                    crate::util::DEFAULT_DRAW_INDEXED_INDIRECT;
                    count
                ]).unwrap();
                
                // Extend the position scaling buffer
                memory.generated_position_scaling_buffers[allocation].extend_from_slice(&vec![vek::Vec4::zero(); count]).unwrap();
                memory.culled_position_scaling_buffers[allocation].extend_from_slice(&vec![vek::Vec4::zero(); count]).unwrap();

                // Extend the visibility vector and buffer
                memory.visibility_buffers[allocation].extend_from_slice(&vec![0; count]).unwrap();
                memory.visibility_bitsets[allocation].reserve(count);

                // Increase the max count of the mesh that corresponds to this allocation
                let handle = &memory.allocation_meshes[allocation];
                *multi_draw_indirect_count_meshes.get_mut(&handle).max_count_mut() += count;

                // Create new chunk entities and set them as "free"
                entities.extend((0..count).into_iter().map(|i| {
                    let position = Position::default();
                    let scale = Scale::default();
                    
                    // Create the chunk component
                    let chunk = Chunk {
                        state: ChunkState::Free,
                        allocation,
                        global_index: global_index, 
                        local_index: old_per_allocation + i,
                        generation_priority: 0.0f32,
                        readback_priority: 0.0f32,
                        ranges: None,
                        node: None,
                    };
                    global_index += 1;
                
                    // Create the bundle
                    (position, scale, chunk)
                }));
            }
        
            // Randomly order the entities to reduce the chances of an OOM error
            entities.shuffle(&mut rng);
            scene.extend_from_iter(entities);
            manager.chunks_per_allocation += count;
        }   

        // Get all free chunks in the world and use them
        let query = scene
            .query_mut::<(
                &mut Chunk,
                &mut Position,
                &mut Scale,
                &Entity,
            )>()
            .into_iter()
            .filter(|(x, _, _, _)| x.state == ChunkState::Free)
            .collect::<Vec<_>>();

        // Set the "dirty" state for newly added chunks
        assert!(query.len() >= added.len());
        for ((chunk, position, scale, entity), node) in query.into_iter().zip(added.iter()) {
            chunk.state = ChunkState::Dirty;
            
            // Set node, position, and scale
            chunk.node = Some(*node);
            **position = node.position().as_::<f32>();
            **scale = (node.size() as f32) / (settings.size as f32);
            
            // Add the entity to the internally stored entities
            let res = manager.entities.insert(*node, *entity);
            memory.visibility_bitsets[chunk.allocation].remove(chunk.local_index);
            assert!(res.is_none());
        }
    }

    for (chunk, position) in scene.query_mut::<(&mut Chunk, &Position)>() {
        // Update generation priority for EACH chunk, even if the viewer did not move
        chunk.generation_priority = (1.0 / viewer_position.distance(**position).max(1.0)) * 10.0;
        chunk.generation_priority *= viewer_rotation
            .forward()
            .dot((**position - viewer_position).normalized())
            * 5.0;
            chunk.generation_priority = chunk.generation_priority.clamp(0.0f32, 1000.0f32);

        // Update readback priority for each chunk *around* the user (needed for collisions)
        chunk.readback_priority = 1.0 / viewer_position.distance(**position).max(1.0);
    }
}

// Adds/removes the chunk entities from the world
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system)
        .after(user);
}
