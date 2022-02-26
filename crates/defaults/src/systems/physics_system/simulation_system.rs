use world::ecs::event::EventKey;
use world::World;

// Run the physics simulation
fn run(world: &mut World, mut data: EventKey) {
    world.physics.step();
}

// Create the physics simulation system
pub fn system(world: &mut World) {
    world.ecs.build_system().with_run_event(run).build();
}
