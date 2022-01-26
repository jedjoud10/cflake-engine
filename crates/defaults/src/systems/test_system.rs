use ecs::component::*;
use main::core::{Context, WriteContext};
use main::ecs;

// Some global data for the test system
pub(crate) struct TestSystemData {}

ecs::impl_component!(TestSystemData);

// A simple system that we can use for testing
fn run(mut context: Context, query: ComponentQuery) {
    let mut write = context.write();
}

// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().set_run_event(run).build();
}
