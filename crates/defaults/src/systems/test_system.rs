use ecs::component::*;
use main::core::{Context, WriteContext};
use main::ecs;

// A simple system that we can use for testing
fn run(_context: &mut Context, _query: ComponentQuery) {
}

// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).build();
}
