use main::{ecs::component::ComponentQuery, core::{Context, WriteContext}, terrain::DEFAULT_LOD_FACTOR};

// The chunk systems' update loop
fn run(mut context: Context, query: ComponentQuery) {
    // Get the global terrain component
    let mut write = context.write();
    // Get the camera position
    let camera_pos = write.ecs.global::<crate::globals::GlobalWorldData>().unwrap().camera_pos;
    dbg!(camera_pos);
    let terrain = write.ecs.global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // Generate the chunks if needed
        let octree = &mut terrain.octree;

        if let Some((added, removed)) = octree.generate_incremental_octree(&camera_pos, DEFAULT_LOD_FACTOR) {
            // We have moved, thus the chunks need to be regenerated

        }
    }
}
// Create a chunk system 
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .build()
}