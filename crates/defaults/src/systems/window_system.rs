use world::ecs::component::ComponentQuerySet;
use world::input::Keys;
use world::World;

// The window system's update loop
fn run(world: &mut World, _data: ComponentQuerySet) {
    if world.input.changed("toggle_fullscreen") {
        world.pipeline.window_mut().set_fullscreen(world.input.toggled("toggle_fullscreen"));
    }
    if world.input.changed("toggle_input") {
        // If "var" is true, we show the cursor
        let var = world.input.toggled("toggle_input");
        world.pipeline.window().context().window().set_cursor_grab(!var).unwrap();
        world.pipeline.window().context().window().set_cursor_visible(var);
    }
}

// Create a system that'll allow us to disable/enable fullscreen
pub fn system(world: &mut World) {
    world.ecs.systems.builder(&mut world.events.ecs).event(run).build().unwrap();
    world.input.bind_toggle(Keys::F2, "toggle_input");
    world.input.bind_toggle(Keys::F5, "toggle_fullscreen");
}
