use main::{
    core::{Context, WriteContext},
    ecs::{component::ComponentQuery, entity::ComponentLinkingGroup},
    rendering::{basics::model::Model, pipeline::pipec}, terrain::mesher::{Mesher, MesherSettings},
};

// The mesher systems' update loop
fn run(context: &mut Context, query: ComponentQuery) {
    let mut write = context.write().unwrap();
    // Get the pipeline without angering the borrow checker
    let pipeline_ = write.pipeline.clone();
    let pipeline = pipeline_.read();

    let terrain = write.ecs.get_global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // For each chunk that has a valid voxel data, we must create it's mesh
        query.update_all(|components| {
            let mut chunk = components.get_component_mut::<crate::components::Chunk>().unwrap();     
            let id = components.get_entity_id().unwrap();       
            if !terrain.mesh_gen_chunk_id.map_or(false, |x| x == id) { return; }
            // We have created voxel data for this chunk, and it is valid
            if chunk.pending_model && chunk.buffered_model.is_none() && !chunk.added_renderer {
                terrain.mesh_gen_chunk_id.take().unwrap();
                // I guess we should create the model now
                let coords = chunk.coords;
                let voxel_data = &terrain.stored_chunk_voxel_data;
                let mesher = Mesher::new(coords, voxel_data, MesherSettings {
                    interpolation: true,
                    skirts: false,
                });
                let model = mesher.build();

                // Construct the model and add it to the chunk entity
                let model_id = pipec::construct(model, &*pipeline);
                chunk.buffered_model = Some(model_id);

                // Create a linking group that contains the renderer
                let id = components.get_entity_id().unwrap();
                chunk.added_renderer = true;
                let mut group = ComponentLinkingGroup::default();
                let renderer = main::rendering::basics::renderer::Renderer::new(true).set_model(model_id).set_material(terrain.material);
                group.link(crate::components::Renderer::new(renderer)).unwrap();
                write.ecs.link_components(id, group).unwrap();
                terrain.chunks_generating.remove(&coords);
            }
        })
    }
}
// Create a mesher system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .with_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Chunk>()
        .build()
}
