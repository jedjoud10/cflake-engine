use ecs::Scene;
use graphics::{ComputePass, Graphics};
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
    let (manager, memory, culler) = (
        &terrain.manager,
        &terrain.memory,
        &terrain.culler,
    );

    // Create compute pass that will cull the indexed indirect buffers
    let graphics = world.get::<Graphics>().unwrap();
    let mut pass = ComputePass::begin(&graphics);
    //let mut active = pass.bind_shader(culler.compute_cull);

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
