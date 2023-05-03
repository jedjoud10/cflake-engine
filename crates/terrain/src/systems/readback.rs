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
    
    let (manager, voxelizer, mesher, memory, settings) = (
        &terrain.manager,
        &terrain.voxelizer,
        &terrain.mesher,
        &terrain.memory,
        &terrain.settings,
    );

    let Some(entity) = manager.last_chunk_generated else {
        return;
    };

    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut entry = scene.entry_mut(entity).unwrap();
    let (chunk, surface) = entry.as_query_mut::<(&mut Chunk, &mut Surface<TerrainMaterial>)>().unwrap();

    let index = 1 - (time.frame_count() as usize % 2);
    let counters = &memory.counters[index];
    let offsets = &memory.offsets[index];
    let voxels = &voxelizer.voxel_textures[index];

    // Readback the counters asynchronously
    counters.async_read(.., |counters| {
        let vertex_count = counters[0];
        let triangle_count = counters[1];    
    }).unwrap();

    // Readback the offsets asynchronously
    let tex_coords_per_sub_allocation = settings.tex_coords_per_sub_allocation;
    let triangles_per_sub_allocation = settings.triangles_per_sub_allocation;
    offsets.async_read(.., move |offsets| {
        let vertices_offset = offsets[0];
        let triangle_indices_offset = offsets[1];

        // Check if we are OOM lol
        if vertices_offset / tex_coords_per_sub_allocation
            != triangle_indices_offset / triangles_per_sub_allocation
        {
            panic!("Out of memory xD MDR");
        }
    }).unwrap();

    /*

    // Read as vertex and triangle separately




    // Calculate sub-allocation index and length
    let count = f32::max(
        vertex_count as f32 / settings.tex_coords_per_sub_allocation as f32,
        triangle_count as f32 / settings.triangles_per_sub_allocation as f32,
    );
    let count = count.ceil() as u32;
    let offset = vertices_offset / settings.tex_coords_per_sub_allocation;

    // Update chunk range (if valid) and set visibility
    if count > 0 {
        chunk.ranges = Some(vek::Vec2::new(offset, count + offset));
        surface.visible = true;
    } else {
        chunk.ranges = None;
        surface.visible = false;
    }
    */
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(crate::systems::manager::system)
        .before(crate::systems::generation::system)
        .before(rendering::systems::rendering::system);
}
