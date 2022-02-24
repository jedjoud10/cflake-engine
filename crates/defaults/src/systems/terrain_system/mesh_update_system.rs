use main::{core::World, ecs::event::EventKey};

// The mesher systems' update loop
fn run(world: &mut World, mut data: EventKey) {
    let query = data.as_query_mut().unwrap();
    for (_, components) in query.write().iter_mut() {
        let mut chunk = components
            .get_component_mut::<crate::components::Chunk>()
            .unwrap();
        // Try to get the updated mesh ID
        let model_id = chunk.updated_model_id.take();
        if let Some(model_id) = model_id {
            let mut renderer = components
                .get_component_mut::<crate::components::Renderer>()
                .unwrap();
            // Update the renderer
            renderer.update_model(&world.pipeline.read(), model_id);
        }
    }
}
// Create a mesh update system that will detect whenever we need to update the mesh ID of a specific chunk and update it accordingly
pub fn system(world: &mut World) {
    world
        .ecs
        .build_system()
        .with_run_event(run)
        .link::<crate::components::Chunk>()
        .link::<crate::components::Renderer>()
        .build()
}
