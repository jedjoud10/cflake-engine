use world::{ecs::component::ComponentQuerySet, World};

// Update the GUI
fn run(world: &mut World, mut _data: ComponentQuerySet) {
    world.gui.draw_frame(&mut world.pipeline);
}

// Create the GUI system
pub fn system(world: &mut World) {
    world.ecs.systems.builder(&mut world.events.ecs).event(run).build();
}
