use main::core::World;
use main::ecs::event::EventKey;
use main::input::Keys;

// The window system's update loop
fn run(world: &mut World, _data: EventKey) {
    let pipeline = world.pipeline.read();
    if world.input.map_changed("toggle_fullscreen") {
        pipeline.window.set_fullscreen(world.input.map_toggled("toggle_fullscreen"));
    }
}

// Create a system that'll allow us to disable/enable fullscreen and vsync
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().with_run_event(run).build();
    world.input.bind_key_toggle(Keys::F5, "toggle_fullscreen");
}
