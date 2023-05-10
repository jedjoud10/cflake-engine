use ecs::Scene;
use rendering::Surface;
use utils::Time;
use world::{System, World};

use crate::{Terrain, Chunk, TerrainMaterial};

// This will iterate over the generated indexed indirect buffers and cull the chunks that are not visible
// The culling will be based on frustum culling and simple visiblity (flag) culling
fn update(world: &mut World) {
    let Ok(mut terrain) = world.get_mut::<Terrain>() else {
        return;
    };
    
    // Decompose the terrain into its subresources
    let (manager, voxelizer, mesher, memory, settings) = (
        &terrain.manager,
        &terrain.voxelizer,
        &terrain.mesher,
        &terrain.memory,
        &terrain.settings,
    );

    // Start render pass for the culling algorithm

    // Create compute pass that will cull the indexed indirect buffers
    // Run the culling compute shader with the data from the camera
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .after(crate::systems::manager::system)
        .after(crate::systems::generation::system)
        .before(rendering::systems::rendering::system);
}
