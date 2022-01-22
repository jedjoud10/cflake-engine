use main::{ecs::component::ComponentQuery, core::{Context, WriteContext}};

// The chunk systems' update loop
fn run(context: Context, query: ComponentQuery) {
    // This is ran for every terrain entity that we have
    query.update_all(|components| {
        let terrain = components.component_mut::<crate::components::Terrain>().unwrap();
    })
}
// Create a chunk system 
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .link::<crate::components::Terrain>()
        .build()
}