use world::ecs::event::EventKey;
use world::input::Keys;
use world::World;

// The window system's update loop
fn run(world: &mut World, _data: EventKey) {
    if world.input.map_changed("toggle_fullscreen") {
        world.pipeline.window.set_fullscreen(world.input.map_toggled("toggle_fullscreen"));
    }
    if world.input.map_changed("toggle_input") {
        // If "var" is true, we show the cursor
        let var = world.input.map_toggled("toggle_input");
        world.pipeline.window.context().window().set_cursor_grab(!var).unwrap();
        world.pipeline.window.context().window().set_cursor_visible(var);
    }
}

// Create a system that'll allow us to disable/enable fullscreen and vsync
pub fn system(world: &mut World) {
    world.ecs.systems.builder().with_run_event(run).build();
    world.input.bind_key_toggle(Keys::F5, "toggle_fullscreen");
    world.input.bind_key_toggle(Keys::F2, "toggle_input");
}
