use main::core::World;
use main::ecs::event::EventKey;

// A simple system that we can use for testing
fn run(_world: &mut World, _data: EventKey) {}

// Create the system
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().with_run_event(run).build();
}
