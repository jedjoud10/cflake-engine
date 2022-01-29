use main::{
    core::{Context, WriteContext},
    ecs::component::ComponentQuery,
    input::Keys,
};

// The debugging system's update loop
fn run(context: &mut Context, _query: ComponentQuery) {
    // Check if we need to debug
    let read = context.read();
    let pipeline = read.pipeline.read();
    if read.input.map_pressed("debug") {
        // Debug some data
        println!("Component count: '{}'", read.ecs.count_components());
        println!("Entity count: '{}'", read.ecs.count_entities());
        println!("System count: '{}'", read.ecs.count_systems());
        println!("Time: '{}', Delta Time: '{}', FPS: '{}'", read.time.elapsed, read.time.delta, 1.0 / read.time.delta);
        main::rendering::pipeline::pipec::set_debugging(true, &*pipeline);
    } else {
        main::rendering::pipeline::pipec::set_debugging(false, &*pipeline);
    }
}
// Create the debugging system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().set_run_event(run).build();
    // Set some debugging keybinds
    write.input.bind_key(Keys::F1, "debug");
}
