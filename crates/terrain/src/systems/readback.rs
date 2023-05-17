use ecs::{Scene, Entity};
use rendering::Surface;
use utils::Time;
use world::{System, World};

use crate::{Terrain, Chunk, TerrainMaterial, ChunkState};

// Begins the async readback of range data at the start of the frame
fn readback_begin_update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let Ok(mut terrain) = world.get_mut::<Terrain>() else {
        return;
    };
    
    // Decompose the terrain into its subresources
    let mut _terrain = terrain;
    let terrain = &mut *_terrain;
    let (manager, voxelizer, mesher, memory, settings) = (
        &mut terrain.manager,
        &terrain.voxelizer,
        &terrain.mesher,
        &mut terrain.memory,
        &terrain.settings,
    );

    // Start doing an async readback for the chunk of last frame
    let last_chunk_generated = scene
        .query_mut::<(&mut Chunk, &Entity)>()
        .into_iter()
        .filter(|(chunk, _)| chunk.state == ChunkState::PendingReadbackStart)
        .next();
    if let Some((chunk, &entity)) = last_chunk_generated {
        chunk.state = ChunkState::PendingReadbackData;
        let index = 1 - (time.frame_count() as usize % 2);
        let counters = &memory.counters[index];
        let offsets = &memory.offsets[index];
        let offset_sender = memory.readback_offset_sender.clone();
        let count_sender = memory.readback_count_sender.clone();
    
        // Readback the counters asynchronously
        counters.async_read(.., move |counters| {
            let _ = count_sender.send((entity, vek::Vec2::from_slice(counters)));
        }).unwrap();
    
        // Readback the offsets asynchronously
        offsets.async_read(.., move |offsets| {
            let _ = offset_sender.send((entity, vek::Vec2::from_slice(offsets)));
        }).unwrap();
    };

    // Fetch multiple at the same time if needed and cache them
    let offset = memory.readback_offset_receiver.try_iter();
    let count = memory.readback_count_receiver.try_iter();
    memory.readback_offsets.extend(offset);
    memory.readback_counters.extend(count);

    // Sort by entity ID and fetch the last one
    memory.readback_offsets.sort_by(|(a, _), (b, _)| Entity::cmp(&b, &a));
    memory.readback_counters.sort_by(|(a, _), (b, _)| Entity::cmp(&b, &a));
}

// At the end of the frame, right before culling
// This will handle the data that was readback from the callbacks
// The data isn't necessarily a single frame delayed, it could be 2 frames or even 3 frames delayed
fn readback_end_update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let Ok(mut terrain) = world.get_mut::<Terrain>() else {
        return;
    };
    
    // Decompose the terrain into its subresources
    let mut _terrain = terrain;
    let terrain = &mut *_terrain;
    let (manager, voxelizer, mesher, memory, settings) = (
        &mut terrain.manager,
        &terrain.voxelizer,
        &terrain.mesher,
        &mut terrain.memory,
        &terrain.settings,
    );

    // TODO: Convert the vecs to hashmaps so we can use entities with the same entity ID

    
    // Fetch the last one (to check if they are the same)
    let offset = memory.readback_offsets.last();
    let count = memory.readback_counters.last();

    // Mismatched entities, tell the chunk to refetch the data
    if let (Some((e1, _)), Some((e2, _))) = (offset, count) {
        if e1 != e2 {
            log::error!("Mismatched entity");
            return;
        }
    }

    let offset = memory.readback_offsets.pop();
    let count = memory.readback_counters.pop();
    if let (Some((e1, offset)), Some((e2, count))) = (offset, count) {
        assert_eq!(e1, e2);

        // Fetch the appropriate chunk
        let mut entry = scene.entry_mut(e1).unwrap();
        let chunk = entry.get_mut::<Chunk>().unwrap();    

        // Check if we are OOM lol
        let vertices_per_sub_allocation = settings.vertices_per_sub_allocation;
        let triangles_per_sub_allocation = settings.triangles_per_sub_allocation;
        if offset.x >= (u32::MAX - vertices_per_sub_allocation + 1) || offset.y >= (u32::MAX - triangles_per_sub_allocation + 1) {
            panic!("Out of memory xD MDR");
        }

        // Calculate sub-allocation index and length
        let count = f32::max(
            count.x as f32 / vertices_per_sub_allocation as f32,
            count.y as f32 / triangles_per_sub_allocation as f32,
        ).ceil() as u32;
        let offset = offset.x / vertices_per_sub_allocation;

        // Update chunk range (if valid) and set visibility
        let filled = count > 0;
        chunk.state = ChunkState::Generated { empty: !filled };
        if filled {            
            chunk.ranges = Some(vek::Vec2::new(offset, count + offset));
        } else {
            chunk.ranges = None;
        }

        // Updates the "generated" count of our parent node if any
        let node = chunk.node.unwrap();
        let parent = manager.octree.nodes().get(node.parent().unwrap()).unwrap();

        // Makes sure we are the proper child of the parent node
        let base = parent.children().unwrap().get();
        assert!((base + 8) > node.index() && node.index() >= base);

        // Don't show the chunks by default, wait until their parent allow us to 
        if let Some((count, entities)) = manager.counting.get_mut(&parent.center()) {
            *count += 1;
            entities.push(e1);
        } else {
            // If we don't have a parent then this chunk is a leaf node of a previous parent node
            // so we must wait till the node generates to be able to get rid of the children
            memory.visibility_bitsets[chunk.allocation].set(chunk.local_index);
        }
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