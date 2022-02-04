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
        println!("Component count: '{}'", read.ecs.count_components());
        println!("Entity count: '{}'", read.ecs.count_entities());
        println!("System count: '{}'", read.ecs.count_systems());
        println!("Time: '{}', Delta Time: '{}', FPS: '{}'", read.time.elapsed, read.time.delta, 1.0 / read.time.delta);
        // Debug some global info
        let global_fetcher = data.get_global_fetcher().unwrap();
        let core_global = read.ecs.get_global::<crate::globals::GlobalWorldData>(&global_fetcher).unwrap();
        println!("Main Camera Position: '{:?}'", core_global.camera_pos);
        main::rendering::pipeline::pipec::set_debugging(true, &*pipeline);
    } else {
        main::rendering::pipeline::pipec::set_debugging(false, &*pipeline);
    }
}
// Create the debugging system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).build();
    // Set some debugging keybinds
    write.input.bind_key(Keys::F1, "debug");
    write.input.bind_key(Keys::F2, "placeholder");
}
