use world::{World, ecs::event::EventKey};

// The lights system update loop
fn run(world: &mut World, mut data: EventKey) {
    world.gui.draw_frame(&mut world.pipeline);
}

// Create the GUI system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .with_run_event(run)
        .build();
}
