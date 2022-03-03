use world::{rendering::pipeline::pipec, World};

// Create the GUI system
pub fn system(world: &mut World) {
    world.ecs.systems.builder().build();
    // We must create the pipeline End of Frame callback and tell it to render our GUI
    let painter = world.gui.painter.clone();
    let pipeline = world.pipeline.read();
    pipec::add_end_of_frame_callback(&pipeline, move |pipeline, _| {
        // Draw the GUI
        let mut painter = painter.lock();
        painter.draw_gui(pipeline);
    });
}
