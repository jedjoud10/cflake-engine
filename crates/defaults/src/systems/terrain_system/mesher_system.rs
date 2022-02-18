use main::{
    core::World,
    ecs::{entity::ComponentLinkingGroup, event::EventKey},
    rendering::pipeline::pipec,
    terrain::mesher::{Mesher, MesherSettings},
};

// The mesher systems' update loop
fn run(world: &mut World, mut data: EventKey) {
    let query = data.as_query_mut().unwrap();
    // Get the pipeline without angering the borrow checker
    let pipeline = world.pipeline.read();
    let terrain = world.globals.get_global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // For each chunk that has a valid voxel data, we must create it's mesh
        for (id, components) in query.lock().iter_mut() {
            let mut chunk = components.get_component_mut::<crate::components::Chunk>().unwrap();
            if !terrain.chunk_handler.mesh_gen_chunk_id.map_or(false, |x| x == *id) {
                continue;
            }
            // We have created voxel data for this chunk, and it is valid
            if chunk.pending_model && chunk.buffered_model.is_none() && !chunk.added_renderer {
                terrain.chunk_handler.mesh_gen_chunk_id.take().unwrap();
                // I guess we should create the model now
                let coords = chunk.coords;
                let voxel_data = &terrain.generator.stored_chunk_voxel_data;
                let mesher = Mesher::new(
                    coords,
                    voxel_data,
                    MesherSettings {
                        interpolation: true,
                        skirts: true,
                    },
                );
                let model = mesher.build();

                // Construct the model and add it to the chunk entity
                let model_id = pipec::construct(&pipeline, model).unwrap();
                chunk.buffered_model = Some(model_id);

                // Create a linking group that contains the renderer
                chunk.added_renderer = true;
                let mut group = ComponentLinkingGroup::default();
                let renderer = main::rendering::basics::renderer::Renderer::new(main::rendering::basics::renderer::RendererFlags::DEFAULT)
                    .with_model(model_id)
                    .with_material(terrain.chunk_handler.material);
                group.link(crate::components::Renderer::new(renderer)).unwrap();
                world.ecs.link_components(*id, group).unwrap();
                terrain.chunk_handler.chunks_generating.remove(&coords);
            }
        }
    }
}
// Create a mesher system
pub fn system(world: &mut World) {
    world
        .ecs
        .create_system_builder()
        .with_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Chunk>()
        .build()
}
