use main::core::World;
use main::ecs::event::EventKey;
use main::input::Keys;

// The window system's update loop
fn run(world: &mut World, _data: EventKey) {
    let pipeline = world.pipeline.read();
    if world.input.map_changed("toggle_fullscreen") {
        pipeline
            .window
            .set_fullscreen(world.input.map_toggled("toggle_fullscreen"));
    }
    if world.input.map_changed("toggle_input") {
        // If "var" is true, we show the cursor
        let var = world.input.map_toggled("toggle_input");
        pipeline
            .window
            .inner
            .as_ref()
            .unwrap()
            .set_cursor_grab(!var)
            .unwrap();
        pipeline
            .window
            .inner
            .as_ref()
            .unwrap()
            .set_cursor_visible(var);
        world.input.accepts_input = !var;
    }
}

// Create a system that'll allow us to disable/enable fullscreen and vsync
pub fn system(world: &mut World) {
    world.ecs.build_system().with_run_event(run).build();
    world.input.bind_key_toggle(Keys::F5, "toggle_fullscreen");
    world.input.bind_key_toggle(Keys::F2, "toggle_input");
}
