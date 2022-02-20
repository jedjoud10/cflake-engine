use main::{
    core::World,
    ecs::{entity::ComponentLinkingGroup, event::EventKey},
    rendering::pipeline::pipec,
    terrain::{mesher::{Mesher, MesherSettings}, StoredVoxelData},
};
use crate::{globals::ChunkGenerationState, components::Chunk};

// A post generation event that will be called after the generation of a specific chunk
fn chunk_post_gen(_terrain: &crate::globals::Terrain, _chunk: &Chunk, _data: &StoredVoxelData) {
}

// The mesher systems' update loop
fn run(world: &mut World, mut data: EventKey) {
    let query = data.as_query_mut().unwrap();
    let terrain = world.globals.get_global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // For each chunk that has a valid voxel data, we must create it's mesh
        for (id, components) in query.write().iter_mut() {
            let chunk = components.get_component_mut::<crate::components::Chunk>().unwrap();
            if terrain.chunk_handler.current_chunk_state == ChunkGenerationState::EndVoxelDataGeneration(*id, true) {            
                // We have created voxel data for this chunk, and it is valid (it contains a surface)
                // I guess we should create the model now
                let coords = chunk.coords;
                let voxel_data = &terrain.voxel_generator.stored_chunk_voxel_data;
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
                // Get the pipeline without angering the borrow checker
                let pipeline = world.pipeline.read();
                let model_id = pipec::construct(&pipeline, model).unwrap();
                drop(pipeline);

                // Create a linking group that contains the renderer
                let mut group = ComponentLinkingGroup::default();
                let renderer = main::rendering::basics::renderer::Renderer::new(main::rendering::basics::renderer::RendererFlags::DEFAULT)
                    .with_model(model_id)
                    .with_material(terrain.chunk_handler.material);
                group.link(crate::components::Renderer::new(renderer)).unwrap();
                world.ecs.link_components(*id, group).unwrap();
                terrain.chunk_handler.chunks_generating.remove(&coords);
                // Switch states
                terrain.chunk_handler.current_chunk_state = ChunkGenerationState::RequiresVoxelData;
                chunk_post_gen(&terrain, &chunk, &terrain.voxel_generator.stored_chunk_voxel_data);
            } else if terrain.chunk_handler.current_chunk_state == ChunkGenerationState::EndVoxelDataGeneration(*id, false) {
                // The chunk ID is the same, but we do not have a surface
                // We still gotta update the current chunk state though
                terrain.chunk_handler.current_chunk_state = ChunkGenerationState::RequiresVoxelData;
                chunk_post_gen(&terrain, &chunk, &terrain.voxel_generator.stored_chunk_voxel_data);
            } else {
                // Skip since this is not the proper chunk
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
