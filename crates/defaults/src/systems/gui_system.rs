use main::{core::World, rendering::pipeline::pipec};

// Create the GUI system
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().build();
    // We must create the pipeline End of Frame callback and tell it to render our GUI
    let painter = world.gui.painter.clone();
    let pipeline = world.pipeline.read();
    pipec::add_end_of_frame_callback(&pipeline, move |pipeline, renderer| {
        // Draw the GUI
        let mut painter = painter.lock().unwrap();
        painter.draw_gui(pipeline, renderer);
    })
    .unwrap();
}
