use main::{
    core::{Context, WriteContext},
    ecs::{component::ComponentQuery, self, entity::EntityID},
    terrain::{DEFAULT_LOD_FACTOR, ChunkCoords}, rendering::basics::texture::TextureReadBytes,
};

// The voxel systems' update loop
fn run(mut context: Context, query: ComponentQuery) {
    let mut write = context.write();
    let terrain = write.ecs.global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        let pipeline = write.pipeline.read().unwrap();
    }
}
// Create a voxel system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Chunk>()
        .build()
}
