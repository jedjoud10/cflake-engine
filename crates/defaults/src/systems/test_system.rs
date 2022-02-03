use main::core::{Context, WriteContext};
use main::ecs::event::EventKey;

// A simple system that we can use for testing
fn run(_context: &mut Context, _data: EventKey) {}

// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).build();
}
