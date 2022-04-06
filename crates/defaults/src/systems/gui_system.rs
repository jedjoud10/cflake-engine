use world::World;

// Draw the GUI
pub fn system(world: &mut World) {
    world.events.insert(|world| {
        world.gui.draw_frame(&mut world.pipeline);
    });
}
