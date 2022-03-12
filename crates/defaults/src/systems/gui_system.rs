use world::{ecs::component::ComponentQuerySet, World};

// The lights system update loop
fn run(world: &mut World, _data: ComponentQuerySet) {
    world.gui.draw_frame(&mut world.pipeline);
}

// Create the GUI system
pub fn system(world: &mut World) {
    world.ecs.systems.builder().event(run).build();
}
