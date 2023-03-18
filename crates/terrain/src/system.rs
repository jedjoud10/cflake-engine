use assets::Assets;
use graphics::Graphics;
use world::{System, World};

use crate::VoxelGenerator;

// Called at the start of the app to add the resources
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let assets = world.get::<Assets>().unwrap();

    // Create the voxel generator
    let voxel = VoxelGenerator::new(&graphics, &assets);

    // Add the resources to the world
    drop(graphics);
    drop(assets);
    world.insert(voxel);
}


// Called each frame before rendering to generate the required voxel data and mesh data for each chunk
fn update(world: &mut World) {

}


// Responsible for terrain generation and rendering
pub fn system(system: &mut System) {
    system.insert_init(init);
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system);
}