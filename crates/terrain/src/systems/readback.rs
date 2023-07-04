use ahash::AHashMap;
use ecs::{Entity, Scene};

use physics::{RigidBody, MeshCollider};
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
    let (_manager, _voxelizer, mesher, memory, settings) = (
        &mut terrain.manager,
        &terrain.voxelizer,
        &terrain.mesher,
        &mut terrain.memory,
        &terrain.settings,
    );

    // Start doing a counter and offset async readback for the chunk of last frame
    let mut last_chunk_generated = scene
        .query_mut::<(&mut Chunk, &Entity)>()
        .into_iter().find(|(chunk, _)| chunk.state == ChunkState::PendingReadbackStart);
    if let Some((chunk, &entity)) = last_chunk_generated.as_mut() {
        chunk.state = ChunkState::PendingReadbackData;
        let counters = &memory.counters;
        let offsets = &memory.offsets;
        
        // Readback the counters asynchronously
        let hashmap = memory.readback_offsets_and_counters.clone();
        counters
            .async_read(.., move |counters| {
                let mut locked = hashmap.lock();
                let (_, out) = locked.entry(entity).or_default();
                *out = Some(vek::Vec2::from_slice(counters));
            })
            .unwrap();

        // Readback the offsets asynchronously
        let hashmap = memory.readback_offsets_and_counters.clone();
        offsets
            .async_read(.., move |offsets| {
                let mut locked = hashmap.lock();
                let (out, _) = locked.entry(entity).or_default();
                *out = Some(vek::Vec2::from_slice(offsets));
            })
            .unwrap();
    };
    
    
    // Fetch any given chunk that we can readback a mesh for (collisions only)
    if settings.mesher.collisions {
        let mesh_readback_chunk = if let Some((chunk, entity)) = last_chunk_generated.as_mut() {
            if chunk.collider {
                Some((chunk, &**entity))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((chunk, &entity)) = mesh_readback_chunk {
            chunk.mesh_readback_state = Some(MeshReadbackState::PendingReadbackData);
            let vertices = &mesher.temp_vertices;
            let triangles = &mesher.temp_triangles;

            // Readback the vertices asynchronously
            let hashmap = memory.readback_vertices_and_triangles.clone();
            vertices
                .async_read(.., move |vertices| {
                    let mut locked = hashmap.lock();
                    let (out, _) = locked.entry(entity).or_default();
                    *out = Some(vertices.to_vec());
                })
                .unwrap();

            // Readback the triangles asynchronously
            let hashmap = memory.readback_vertices_and_triangles.clone();
            triangles
                .async_read(.., move |triangles| {
                    let mut locked = hashmap.lock();
                    let (_, out) = locked.entry(entity).or_default();
                    *out = Some(triangles.to_vec());
                })
                .unwrap();
        }
    }
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

    // Find the first entity that has both counters and offsets fetched back
    let mut hashmap = memory.readback_offsets_and_counters.lock();
    let iter = hashmap
        .iter()
        .filter_map(|(e, (a, b))| {
            let a = a.as_ref()?;
            let b = b.as_ref()?;
            Some((*e, *a, *b))
        }).next();

    if let Some((entity, offset, count)) = iter {
        hashmap.remove(&entity).unwrap();

        // Fetch the appropriate chunk
        let mut entry = scene.entry_mut(entity).unwrap();
        let chunk: &mut Chunk = entry.get_mut::<Chunk>().unwrap();

        // Check if we are OOM lol
        let vertices_per_sub_allocation = memory.vertices_per_sub_allocation;
        let triangles_per_sub_allocation = memory.triangles_per_sub_allocation;
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
        let solid = count > 0;
        chunk.state = ChunkState::Generated { empty: !solid };

        // Disable the range and visibility if the mesh is not valid
        if solid {
            manager.new_visibilities.push((chunk.allocation, chunk.local_index));
            chunk.ranges = Some(vek::Vec2::new(offset, count + offset));
        } else {
            chunk.ranges = None;
        }
    }


    // Find the first entity that has both vertices and triangles fetched back
    let mut hashmap = memory.readback_vertices_and_triangles.lock();
    let iter = hashmap
        .iter()
        .filter_map(|(e, (a, b))| {
            a.as_ref()?;
            b.as_ref()?;
            Some(*e)
        }).next();
    if let Some(entity) = iter {
        let (vertices, triangles) = hashmap.remove(&entity).unwrap();
        let vertices = vertices.unwrap();
        let triangles = triangles.unwrap();
        
        let mut entry = scene.entry_mut(entity).unwrap();
        let mut chunk = entry.get_mut::<Chunk>().unwrap();
        let node = chunk.node.unwrap();
        chunk.mesh_readback_state = Some(MeshReadbackState::Complete);
        let collision = entry.get_mut::<MeshCollider>().unwrap();
        
        let vertices = crate::util::transform_vertices(vertices, node);
        collision.set_geometry(vertices, triangles);
    }
}

// Reads back the data from the GPU at the start of the frame
pub fn readback_begin_system(system: &mut System) {
    system
        .insert_update(readback_begin_update)
        .before(crate::systems::manager::system)
        .before(crate::systems::generation::system)
        .after(utils::time)
        .before(rendering::systems::rendering::system);
}

// Handles the readback data for *any* given chunk
pub fn readback_end_system(system: &mut System) {
    system
        .insert_update(readback_end_update)
        .after(crate::systems::manager::system)
        .after(crate::systems::generation::system)
        .before(crate::systems::cull::system)
        .after(utils::time)
        .before(rendering::systems::rendering::system);
}
