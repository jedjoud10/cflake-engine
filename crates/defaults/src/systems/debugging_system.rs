use main::{core::World, ecs::event::EventKey, input::Keys};

// The debugging system's update loop
fn run(world: &mut World, _data: EventKey) {
    // Check if we need to debug
    let pipeline = world.pipeline.read();
    if world.input.map_pressed("debug") {
        // Debug some data

        println!("Time: '{}', Delta Time: '{}', FPS: '{}'", world.time.elapsed, world.time.delta, 1.0 / world.time.delta);
        println!("ECS: ");
        println!("  #Component: '{}'", world.ecs.count_components());
        println!("  #Entities: '{}'", world.ecs.count_entities());
        println!("  #Systems: '{}'", world.ecs.count_systems());
        // Debug some global info
        let core_global = world.globals.get_global::<crate::globals::GlobalWorldData>().unwrap();
        println!("Global: ");
        println!("  #Camera Position: '{}'", core_global.camera_pos);
        main::rendering::pipeline::pipec::set_debugging(&pipeline, true);
        // Also debug the terrain if needed
        let terrain = world.globals.get_global::<crate::globals::Terrain>();
        if let Ok(terrain) = terrain {
            println!("Terrain: ");
            println!("  #Chunk Size: [{a}x{a}x{a}]", a = main::terrain::CHUNK_SIZE);
            println!("  #Chunks: '{}'", terrain.chunks.len());
            println!("  #Pending Generation: '{}'", terrain.chunks_generating.len());
            println!("  #Pending Deletion: '{}'", terrain.chunks_to_remove.len());
        }
    } else {
        main::rendering::pipeline::pipec::set_debugging(&pipeline, false);
    }
}
// Create the debugging system
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().with_run_event(run).build();
    // Set some debugging keybinds
    world.input.bind_key(Keys::F1, "debug");
    world.input.bind_key(Keys::F2, "placeholder");
}
