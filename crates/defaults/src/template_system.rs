use main::core::World;
use main::ecs::event::EventKey;

// A simple system that we can use as template
fn run(_world: &mut World, mut _data: EventKey) {}

// Create the system
pub fn system(world: &mut World) {
    world.ecs.build_system().with_run_event(run).build();
}
