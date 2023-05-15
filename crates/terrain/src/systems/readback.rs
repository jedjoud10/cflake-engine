use ecs::Scene;
use rendering::Surface;
use utils::Time;
use world::{System, World};

use crate::{Terrain, Chunk, TerrainMaterial};

// Reads back the voxel values generated in the last frame
// This will read from the voxel buffer that was used last frame (double-buffered)
fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
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
        &terrain.memory,
        &terrain.settings,
    );

    // Start doing an async readback for the chunk of last frame
    if let Some(entity) = manager.last_chunk_generated.take() {
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

    // Handle the chunk that was readback the frame before
    let mut scene = world.get_mut::<Scene>().unwrap();
    let offset = memory.readback_offset_receiver.try_recv();
    let count = memory.readback_count_receiver.try_recv();
    if let (Ok((e1, offset)), Ok((e2, count))) = (offset, count) {
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
        if count > 0 {            
            chunk.ranges = Some(vek::Vec2::new(offset, count + offset));
        } else {
            chunk.ranges = None;
        }
    }
    
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(crate::systems::manager::system)
        .before(crate::systems::generation::system)
        .after(utils::time)
        .before(rendering::systems::rendering::system);
}
