use main::{ecs::{component::ComponentQuery, entity::ComponentLinkingGroup}, core::{Context, WriteContext}, rendering::{pipeline::pipec, basics::model::Model}};



// The mesher systems' update loop
fn run(mut context: Context, query: ComponentQuery) {
    let mut write = context.write();
    // Get the pipeline without angering the borrow checker
    let pipeline_ = write.pipeline.clone();
    let pipeline = pipeline_.read().unwrap();
    
    let terrain = write.ecs.global_mut::<crate::globals::Terrain>();
    if let Ok(terrain) = terrain {
        // For each chunk that has a valid voxel data, we must create it's mesh
        query.update_all(|components| {
            let mut chunk = components.component_mut::<crate::components::Chunk>().unwrap();
            let id = components.get_entity_id().unwrap();
            // We have created voxel data for this chunk, and it is valid
            let model_id = if chunk.voxel_data.is_some() && chunk.valid_surface && !chunk.valid_model {
                // I guess we should create the model now
                chunk.valid_model = true;
                let voxels = chunk.voxel_data.as_ref().unwrap();
                let coords = chunk.coords;
                let model = main::terrain::mesher::generate_model(voxels, coords, true);

                // Create the actual pipeline model now
                let skirts = model.skirts_model;
                let model = model.model;
                // Combine the models first
                let mut model = Model::combine(model, skirts);
                
                // Make sure the model has all valid field
                model.generate_normals();

                // Construct the model and add it to the chunk entity
                let model_id = pipec::construct(model, &*pipeline);
                Some(model_id)
            } else { None };
            drop(chunk);

            if let Some(model_id) = model_id {
                // Create a linking group that contains the renderer
                let mut group = ComponentLinkingGroup::new();
                let renderer = main::rendering::basics::renderer::Renderer::default().set_model(model_id).set_material(terrain.material);
                group.link(crate::components::Renderer::new(renderer)).unwrap();
                write.ecs.link_components(id, group).unwrap();
            }
        })
    }
}
// Create a mesher system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Chunk>()
        .build()
}
