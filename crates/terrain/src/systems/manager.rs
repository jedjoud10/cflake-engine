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
use rand::Rng;
use rendering::{IndirectMesh, Renderer, Surface, MultiDrawIndirectMesh};
use utils::{Storage, Time};
use world::{user, System, World};

// Dynamically generate the chunks based on camera position
fn update(world: &mut World) {
    // Tries to find a chunk viewer and the terrain generator
    let terrain = world.get_mut::<Terrain>();
    let mut scene = world.get_mut::<Scene>().unwrap();
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

    // Set the main viewer location and fetches the oldvalue
    let mut added = false;
    let new = **viewer_position;
    let old = if let Some((_, old, _)) = &mut manager.viewer {
        std::mem::replace(old, new)
    } else {
        manager.viewer = Some((*entity, new, **viewer_rotation));
        added = true;
        new
    };

    // Check if it moved since last frame
    if added || new != old {
        // Regenerate the octree and detect diffs
        let OctreeDelta {
            mut added,
            removed
        } = manager.octree.compute(new);

        // Discard non-leaf nodes
        added.retain(|x| x.leaf());

        /*
        manager.children_count.clear();
        for node in manager.octree.nodes() {
            if let Some(base) = node.children() {
                let base = base.get();
                let count = &manager.octree.nodes()[base..(base+8)];
                
                manager.children_count.insert(*node, (0, count.into_iter().filter(|x| x.leaf()).count()));
            }
        }
        */

        // Don't do shit
        if added.is_empty() && removed.is_empty() {
            return;
        }

        // If we don't add chunks just exit
        if added.is_empty() {
            return;
        }

        // TODO:
        // Gpu frustum octree culling
        // Gpu visibility check filter
        // Gpu occlusion culling maybe?

        // Set the chunk state to "free" so we can reuse it
        for coord in removed {
            if let Some(entity) = manager.entities.remove(&coord) {
                let mut entry = scene.entry_mut(entity).unwrap();
                
                // Set the chunk as "free" and hide it
                let (chunk, surface) = entry.as_query_mut::<(&mut Chunk, &mut Surface<TerrainMaterial>)>().unwrap();
                chunk.state = ChunkState::Free;
                surface.visible = false;
            }
        }

        // Get the number of free chunks that we can reuse
        let query_count = scene
            .query_mut::<&Chunk>()
            .into_iter()
            .filter(|x| x.state == ChunkState::Free)
            .count();

        // Add extra chunks if we need them
        let mut rng = rand::thread_rng();
        let mut multi_draw_indirect_meshes = world.get_mut::<Storage<MultiDrawIndirectMesh>>().unwrap();
        let mut indexed_indirect_buffers = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
        let indexed_indirect_buffer = indexed_indirect_buffers.get_mut(&memory.indexed_indirect_buffer);

        // Extend the indexed indirect buffer if needed
        if added.len() > query_count {
            // Over-allocate so we don't need to do this as many times
            let chunks_to_pre_allocate = ((added.len() - query_count) * 2).max(128);

            // Global count is the number of chunks that are currently pre-allocated
            // So basically the number of elements within the indexed_indirect_buffer buffer 
            let global_count = indexed_indirect_buffer.len();

            // Extend the indirect draw buffer
            indexed_indirect_buffer.extend_from_slice(&vec![
                DrawIndexedIndirect {
                    vertex_count: 0,
                    instance_count: 1,
                    base_index: 0,
                    vertex_offset: 0,
                    base_instance: 0,
                };
                chunks_to_pre_allocate
            ]).unwrap();

            // Extend the position scaling buffer
            manager.position_scaling_buffer.extend_from_slice(&vec![vek::Vec4::zero(); chunks_to_pre_allocate]).unwrap();

            // Create new chunk entities and set them as "free"
            scene.extend_from_iter((0..chunks_to_pre_allocate).into_iter().map(|index| {
                // Get the allocation index for this chunk
                let allocation = rng.gen_range(0..settings.allocation_count);
                let local_index = memory.chunks_per_allocations[allocation];
                let global_index = global_count + index;
                let position = Position::default();
                let scale = Scale::default();

                // New entity is used by the allocation
                let mesh = multi_draw_indirect_meshes.get_mut(&memory.allocation_meshes[allocation]);
                *mesh.count_mut() += 1;

                // Create the chunk component
                let chunk = Chunk {
                    state: ChunkState::Free,
                    allocation,
                    local_index,
                    global_index,
                    generation_priority: 0.0f32,
                    readback_priority: 0.0f32,
                    ranges: None,
                    node: None,
                };
    
                // Take in account this chunk within the allocation
                memory.chunks_per_allocations[allocation] += 1;
    
                // Create the bundle
                (position, scale, chunk)
            }));
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
            //surface.visible = false;
            
            // Set node, position, and scale
            chunk.node = Some(*node);
            **position = node.position().as_::<f32>();
            **scale = (node.size() as f32) / (settings.size as f32);

            // Update position buffer
            let packed = (*position).with_w(**scale);
            manager.position_scaling_buffer.write(&[packed], chunk.global_index).unwrap();

            // Add the entity to the internally stored entities
            let res = manager.entities.insert(*node, *entity);
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
        chunk.readback_priority = (1.0 / viewer_position.distance(**position).max(1.0));
    }
}

// Adds/removes the chunk entities from the world
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system)
        .after(user);
}
