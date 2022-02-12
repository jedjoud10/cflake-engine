use main::core::World;
use main::rendering::pipeline::pipec;
use std::sync::{Arc, Mutex};

// Create the system
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().build();
    // We must create the pipeline End of Frame callback and tell it to render our UI
    let pipeline = world.pipeline.read();
    let clone = world.ui.ui.clone();
    pipec::add_end_of_frame_callback(&pipeline, move |pipeline, _| {
        
    })
    .unwrap();
}
