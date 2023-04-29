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
use rendering::{IndirectMesh, Renderer, Surface};
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

    // Get the terrain chunk manager and terrain settings
    let terrain = &mut *_terrain;
    let mut manager = &mut terrain.manager;
    let mut memory = &mut terrain.memory;
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
    if added || new != new {
        // Regenerate the octree and detect diffs
        let OctreeDelta {
            mut added,
            mut removed
        } = manager.octree.compute(new, settings.radius);

        // Discard non-leaf nodes
        dbg!(added.len());
        dbg!(removed.len());
        added.retain(|x| x.children().is_none());
        removed.retain(|x| x.children().is_none());
        dbg!(added.len());
        dbg!(removed.len());

        // Don't do shit
        if added.is_empty() && removed.is_empty() {
            return;
        }

        // Set the chunk state to "free" so we can reuse it
        for coord in removed {
            let entity = manager.entities.remove(&coord).unwrap();
            let mut entry = scene.entry_mut(entity).unwrap();
            let chunk = entry.get_mut::<Chunk>().unwrap();
            chunk.state = ChunkState::Free;
        }

        // Try to re-use "free" chunks first 
        let query = scene
            .query_mut::<(
                &mut Chunk,
                &mut Position,
                &mut Scale,
                &Entity,
                &mut Surface<TerrainMaterial>,
            )>()
            .into_iter()
            .filter(|(x, _, _, _, _)| x.state == ChunkState::Free)
            .collect::<Vec<_>>();

        // Add extra chunks if we need them
        let mut rng = rand::thread_rng();
        let mut indirect_meshes = world.get_mut::<Storage<IndirectMesh>>().unwrap();
        let mut indexed_indirect_buffers = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
        let indexed_indirect_buffer = indexed_indirect_buffers.get_mut(&memory.indexed_indirect_buffer);

        // Extend the indexed indirect buffer if needed
        if added.len() > query.len() {
            let count = added.len() - query.len();

            indexed_indirect_buffer.extend_from_slice(&vec![
                DrawIndexedIndirect {
                    vertex_count: 0,
                    instance_count: 1,
                    base_index: 0,
                    vertex_offset: 0,
                    base_instance: 0,
                };
                count
            ]).unwrap();

            log::debug!("Adding {count} new chunk entities to the pool");
        }

        let mut count = 0;
        scene.extend_from_iter((0..(added.len().saturating_sub(query.len()))).into_iter().map(|index| {
            // Get the node that corresponds to this index
            let node = added[index + query.len()];

            // Get the allocation index for this chunk
            let allocation = rng.gen_range(0..settings.allocation_count);
            let local_index = memory.chunks_per_allocations[allocation];
            let global_index = count;
            log::warn!("manager: used allocation {allocation}");
            log::warn!("manager: local index {local_index}");
            log::warn!("manager: global index {global_index}");

            // Get the vertex and triangle buffers that will be shared for this group
            let tex_coord_buffer = &memory.shared_tex_coord_buffers[allocation];
            let triangle_buffer = &memory.shared_triangle_buffers[allocation];

            // Create the indirect mesh
            let mut mesh = IndirectMesh::from_handles(
                None,
                None,
                None,
                Some(tex_coord_buffer.clone()),
                triangle_buffer.clone(),
                memory.indexed_indirect_buffer.clone(),
                count,
            );

            // Set the bounding box of the mesh before hand
            mesh.set_aabb(Some(math::Aabb {
                min: vek::Vec3::zero(),
                max: vek::Vec3::one() * settings.size as f32,
            }));

            // Insert the mesh into the storage
            let mesh = indirect_meshes.insert(mesh);

            // Create the surface for rendering
            let mut surface = Surface::new(
                mesh.clone(),
                manager.material.clone(),
                manager.id.clone()
            );

            // Hide the surface at first
            surface.visible = false;

            // Create a renderer an a position component
            let mut renderer = Renderer::default();
            renderer.instant_initialized = None;
            let position = Position::from(node.position().as_::<f32>());
            let scale = Scale::uniform((node.size() as f32) / (settings.size as f32));

            // Create the chunk component
            let chunk = Chunk {
                state: ChunkState::Dirty,
                allocation,
                local_index,
                global_index,
                priority: 0.0f32,
                ranges: None,
                node,
            };

            count += 1;
            memory.chunks_per_allocations[allocation] += 1;

            // Create the bundle
            (surface, renderer, position, scale, chunk)
        }));

        // Set the "dirty" state for newly added chunks
        for ((chunk, position, scale, entity, surface), node) in query.into_iter().zip(added.iter()) {
            chunk.state = ChunkState::Dirty;
            surface.visible = false;
            
            // Set node, position, and scale
            chunk.node = *node;
            **position = node.position().as_::<f32>();
            **scale = (node.size() as f32) / (settings.size as f32);

            // Add the entity to the internally stored entities
            manager.entities.insert(*node, *entity);
        }
    }

    // Update priority for EACH chunk, even if the viewer did not move
    for (chunk, position) in scene.query_mut::<(&mut Chunk, &Position)>() {
        chunk.priority = (1.0 / viewer_position.distance(**position).max(1.0)) * 10.0;
        chunk.priority *= viewer_rotation
            .forward()
            .dot((**position - viewer_position).normalized())
            * 5.0;
        chunk.priority = chunk.priority.clamp(0.0f32, 1000.0f32);
    }
}

// Adds/removes the chunk entities from the world
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system)
        .after(user);
}
