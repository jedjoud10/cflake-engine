use main::core::World;
use main::rendering::pipeline::pipec;
use std::sync::{Arc, Mutex};

// Create the system
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().build();
    // We must create the pipeline End of Frame callback and tell it to render our UI
    let arc_roots_clone = world.ui.roots.clone();
    let pipeline = world.pipeline.read();
    // We must create the UI renderer on the main thread, but we later send it to the render thread for rendering
    let renderer = Arc::new(Mutex::new(main::ui::Renderer::new(&pipeline)));
    pipec::add_end_of_frame_callback(&pipeline, move |pipeline, _| {
        let mut roots = arc_roots_clone.lock().unwrap();
        let mut renderer = renderer.lock().unwrap();
        // We gotta draw each visible root now
        for (_, root) in &mut *roots {
            renderer.draw(pipeline, root, pipeline.window.dimensions);
        }
    })
    .unwrap();
}
