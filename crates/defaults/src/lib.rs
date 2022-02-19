use main::core::World;
use systems::*;
// Default components
pub mod components;
// Default globals
pub mod globals;
// Default systems
pub mod systems;
pub mod template_system;

// Pre-load the default systems
pub fn preload_system(world: &mut World) {
    //template_system::system(world);
    camera_system::system(world);
    //physics_system::system(world);
    rendering_system::system(world);
    debugging_system::system(world);
    window_system::system(world);
    gui_system::system(world);
    test_system::system(world);
    audio_system::system(world);
    // Terrain
    terrain_system::chunk_system::system(world);
    terrain_system::voxel_system::system(world);
    terrain_system::mesher_system::system(world);

    // We gotta add the globa
    world.globals.add_global(crate::globals::GlobalWorldData::default()).unwrap();
}
