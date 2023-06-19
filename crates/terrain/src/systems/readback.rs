use ahash::AHashMap;
use ecs::{Entity, Scene};

use utils::Time;
use world::{System, World};

use crate::{Chunk, ChunkState, Terrain, MeshReadbackState};

// Begins the async readback of range data at the start of the frame
fn readback_begin_update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let Ok(terrain) = world.get_mut::<Terrain>() else {
        return;
    };

    // Decompose the terrain into its subresources
    let mut _terrain = terrain;
    let terrain = &mut *_terrain;
    let (_manager, _voxelizer, mesher, memory, _settings) = (
        &mut terrain.manager,
        &terrain.voxelizer,
        &terrain.mesher,
        &mut terrain.memory,
        &terrain.settings,
    );

    // Start doing a counter and offset async readback for the chunk of last frame
    let last_chunk_generated = scene
        .query_mut::<(&mut Chunk, &Entity)>()
        .into_iter().find(|(chunk, _)| chunk.state == ChunkState::PendingReadbackStart);
    if let Some((chunk, &entity)) = last_chunk_generated {
        chunk.state = ChunkState::PendingReadbackData;
        let counters = &memory.counters;
        let offsets = &memory.offsets;
        let offset_sender = memory.readback_offset_sender.clone();
        let count_sender = memory.readback_count_sender.clone();

        // Readback the counters asynchronously
        counters
            .async_read(.., move |counters| {
                let _ = count_sender.send((entity, vek::Vec2::from_slice(counters)));
            })
            .unwrap();

        // Readback the offsets asynchronously
        offsets
            .async_read(.., move |offsets| {
                let _ = offset_sender.send((entity, vek::Vec2::from_slice(offsets)));
            })
            .unwrap();
    };

    for (entity, offset) in memory.readback_offset_receiver.try_iter() {
        let (offsets, _) = memory.readback_offsets_and_counters.entry(entity).or_default();
        *offsets = Some(offset);
    }

    for (entity, counter) in memory.readback_count_receiver.try_iter() {
        let (_, counters) = memory.readback_offsets_and_counters.entry(entity).or_default();
        *counters = Some(counter);
    }
    
    // Start doing a mesh async readback for the chunk of last frame
    let last_chunk_generated = scene
        .query_mut::<(&mut Chunk, &Entity)>()
        .into_iter()
        .filter(|(chunk, _)| 
            chunk.readback_priority.is_some()
            && chunk.state == ChunkState::Generated {
                empty: false,
                mesh_readback_state: Some(MeshReadbackState::PendingReadbackStart)
            })
        .next();
    if let Some((chunk, &entity)) = last_chunk_generated {
        if let ChunkState::Generated { mesh_readback_state: Some(mesh_readback_state), .. } = &mut chunk.state {
            *mesh_readback_state = MeshReadbackState::PendingReadbackData;
            
            let vertices = &mesher.temp_vertices;
            let triangles = &mesher.temp_triangles;
            let triangles_sender = memory.readback_triangles_sender.clone();
            let vertices_sender = memory.readback_vertices_sender.clone();
    
            /*
            // Readback the vertices asynchronously
            triangles
                .async_read(.., move |triangles| {
                    let _ = triangles_sender.send((entity, triangles.to_vec()));
                })
                .unwrap();
    
            // Readback the triangles asynchronously
            vertices
                .async_read(.., move |vertices| {
                    let _ = vertices_sender.send((entity, vertices.to_vec()));
                })
                .unwrap();
            */
        }
    };
}

// At the end of the frame, right before culling
// This will handle the data that was readback from the callbacks
// The data isn't necessarily a single frame delayed, it could be 2 frames or even 3 frames delayed
fn readback_end_update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let Ok(terrain) = world.get_mut::<Terrain>() else {
        return;
    };

    // Decompose the terrain into its subresources
    let mut _terrain = terrain;
    let terrain = &mut *_terrain;
    let (manager, memory, settings) = (
        &mut terrain.manager,
        &mut terrain.memory,
        &terrain.settings,
    );

    let iter = memory.readback_offsets_and_counters.iter().filter(|(_, (a, b))| a.is_some() && b.is_some()).map(|(e, _)| e).cloned().next();
    let Some(entity) = iter else {
        return;
    };
    let val = memory.readback_offsets_and_counters.remove(&entity).unwrap();
    let (Some(offset), Some(count)) = val else {
        panic!();
    };

    // Fetch the appropriate chunk
    let mut entry = scene.entry_mut(entity).unwrap();
    let chunk = entry.get_mut::<Chunk>().unwrap();

    // Check if we are OOM lol
    let vertices_per_sub_allocation = settings.vertices_per_sub_allocation;
    let triangles_per_sub_allocation = settings.triangles_per_sub_allocation;
    if offset.x >= (u32::MAX - vertices_per_sub_allocation + 1)
        || offset.y >= (u32::MAX - triangles_per_sub_allocation + 1)
    {
        panic!("Out of memory xD MDR");
    }

    // Calculate sub-allocation index and length
    let count = f32::max(
        count.x as f32 / vertices_per_sub_allocation as f32,
        count.y as f32 / triangles_per_sub_allocation as f32,
    )
    .ceil() as u32;
    let offset = offset.x / vertices_per_sub_allocation;

    // Update chunk range (if valid) and set visibility
    let valid = count > 0;

    // Create the mesh readback state based on our readback priority
    let mesh_readback_state = valid.then_some(MeshReadbackState::PendingReadbackStart);
    chunk.state = ChunkState::Generated { empty: !valid, mesh_readback_state };

    // Disable the range if the mesh is not valid
    if valid {
        chunk.ranges = Some(vek::Vec2::new(offset, count + offset));
    } else {
        chunk.ranges = None;
    }

    // Set visibility if the chunk is actually visible
    if valid {
        manager.new_visibilities.push((chunk.allocation, chunk.local_index));
        memory.visibility_bitsets[chunk.allocation].set(chunk.local_index);
    }
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(readback_begin_update)
        .before(crate::systems::manager::system)
        .before(crate::systems::generation::system)
        .after(utils::time)
        .before(rendering::systems::rendering::system);
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system2(system: &mut System) {
    system
        .insert_update(readback_end_update)
        .after(crate::systems::manager::system)
        .after(crate::systems::generation::system)
        .before(crate::systems::cull::system)
        .after(utils::time)
        .before(rendering::systems::rendering::system);
}
