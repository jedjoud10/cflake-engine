use std::sync::{Arc, Mutex};

use main::core::{Context, WriteContext};
use main::ecs::event::EventKey;
use main::rendering::pipeline::pipec;

// We now must render the UI roots on the render thread
fn run(_context: &mut Context, _data: EventKey) {}

// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).build();
    // We must create the pipeline End of Frame callback and tell it to render our UI
    let arc_roots_clone = write.ui.roots.clone();
    let pipeline = write.pipeline.read();
    // We must create the UI renderer on the main thread, but we later send it to the render thread for rendering
    let renderer = Arc::new(Mutex::new(main::ui::Renderer::new(&pipeline)));
    pipec::add_end_of_frame_callback(move |pipeline| {
        let mut roots = arc_roots_clone.lock().unwrap();
        let mut renderer = renderer.lock().unwrap();
        // We gotta draw each visible root now
        for (_, root) in &mut *roots {
            renderer.draw(pipeline, root, pipeline.window.dimensions);  
        }        
    }, &pipeline);
}
