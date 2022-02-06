use main::{
    core::{Context, WriteContext},
    ecs::event::EventKey,
    input::Keys,
};

// The debugging system's update loop
fn run(context: &mut Context, data: EventKey) {
    // Check if we need to debug
    let read = context.read().unwrap();
    let pipeline = read.pipeline.read();
    if read.input.map_pressed("debug") {
        // Debug some data

        println!("Time: '{}', Delta Time: '{}', FPS: '{}'", read.time.elapsed, read.time.delta, 1.0 / read.time.delta);
        println!("ECS: ");
        println!("  #Component: '{}'", read.ecs.count_components());
        println!("  #Entities: '{}'", read.ecs.count_entities());
        println!("  #Systems: '{}'", read.ecs.count_systems());
        // Debug some global info
        let core_global = read.globals.get_global::<crate::globals::GlobalWorldData>().unwrap();
        println!("Global: ");
        println!("  #Camera Position: '{}'", core_global.camera_pos);
        main::rendering::pipeline::pipec::set_debugging(&pipeline, true);
        // Also debug the terrain if needed
        let terrain = read.globals.get_global::<crate::globals::Terrain>();
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
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).build();
    // Set some debugging keybinds
    write.input.bind_key(Keys::F1, "debug");
    write.input.bind_key(Keys::F2, "placeholder");
}
