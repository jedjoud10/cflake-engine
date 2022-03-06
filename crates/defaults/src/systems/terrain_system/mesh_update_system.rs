use world::{ecs::event::EventKey, World};

// The mesher systems' update loop
fn run(world: &mut World, mut data: EventKey) {
    let query = data.as_query_mut().unwrap();
    for (_, components) in query.iter_mut() {
        let mut chunk = components.get_mut::<crate::components::Chunk>().unwrap();
        // Try to get the updated mesh ID
        let mesh_id = chunk.updated_mesh_id.take();
        if let Some(mesh_id) = mesh_id {
            let mut renderer = components.get_mut::<crate::components::Renderer>().unwrap();
            // Update the renderer
            renderer.update_mesh(&world.pipeline.read(), mesh_id);
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
