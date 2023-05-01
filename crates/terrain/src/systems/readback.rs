use utils::Time;
use world::{System, World};

use crate::Terrain;

// Reads back the voxel values generated in the last frame
// This will read from the voxel buffer that was used last frame (double-buffered)
fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let Ok(mut terrain) = world.get_mut::<Terrain>() else {
        return;
    };
    
    let voxelizer = &mut terrain.voxelizer;
    let index = time.frame_count() as usize % 2;
    let voxels = &voxelizer.voxel_textures[1 - index];

    // Perform an asynchronous texture read
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(crate::systems::manager::system)
        .before(crate::systems::generation::system)
        .before(rendering::systems::rendering::system);
}
