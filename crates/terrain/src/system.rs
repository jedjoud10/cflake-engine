use assets::Assets;
use graphics::{ComputePass, Graphics, DrawIndexedIndirectBuffer};
use rendering::Mesh;
use utils::Storage;
use world::{System, World};

use crate::{VoxelGenerator, MeshGenerator};

// Called at the start of the app to add the resources
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let assets = world.get::<Assets>().unwrap();
    let mut indirect = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();

    // Global chunk resolution
    let size = 32;

    // Create the compute generators
    let voxel = VoxelGenerator::new(&graphics, &assets, size);
    let mesh = MeshGenerator::new(&graphics, &assets, &mut indirect, &mut meshes, size);

    // Add the resources to the world
    drop(graphics);
    drop(assets);
    drop(indirect);
    drop(meshes);
    world.insert(voxel);
    world.insert(mesh);
}

// Called each frame before rendering to generate the required voxel data and mesh data for each chunk
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut _voxels = world.get_mut::<VoxelGenerator>().unwrap();
    let voxels = &mut *_voxels;
    let mut indirect = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();

    // Create a compute pass for both the voxel and mesh compute shaders
    let mut pass = ComputePass::begin(&graphics);

    // Create the voxel data and store it in the image
    let mut active = pass.bind_shader(&voxels.shader);
    active.set_bind_group(0, |set| {
        set.set_storage_texture("densities", &mut voxels.densities)
            .unwrap();
    });
    active.dispatch(vek::Vec3::broadcast(voxels.dispatch));

    // Create the mesh every frame (DEBUG)
}

// Responsible for terrain generation and rendering
pub fn system(system: &mut System) {
    system.insert_init(init);
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system);
}
