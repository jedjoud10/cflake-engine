use ecs::component::*;
use main::core::{Context, WriteContext};
use main::ecs;

// A simple system that we can use for testing
fn run(context: &mut Context, _query: ComponentQuery) {
    let mut write = context.write().unwrap();
    let global1 = write.ecs.get_global_mut::<crate::globals::GlobalWorldData>().unwrap();
    let global2 = write.ecs.get_global_mut::<crate::globals::GlobalWorldData>().unwrap();
    dbg!(global1.camera_dir);
}

// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).build();
}
