use world::input::Keys;
use world::World;

// Handle window events like fullscreening or toggling input
fn run(world: &mut World) {
    if world.input.changed("toggle_fullscreen") {
        world.pipeline.window_mut().set_fullscreen(world.input.toggled("toggle_fullscreen"));
    }
    {
        // If "var" is true, we show the cursor
        let var = world.input.toggled("toggle_input");
        world.pipeline.window().context().window().set_cursor_grab(!var).unwrap();
        world.pipeline.window().context().window().set_cursor_visible(var);
        world.input.set_is_accepting_input(!var);
    }
}

// Create a system that'll allow us to disable/enable fullscreen
pub fn system(world: &mut World) {
    world.events.insert(run);
    world.input.bind_toggle(Keys::F2, "toggle_input");
    world.input.bind_toggle(Keys::F5, "toggle_fullscreen");
}
