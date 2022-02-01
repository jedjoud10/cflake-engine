use ecs::component::*;

use main::core::{Context, WriteContext};
use main::ecs;
use main::input::Keys;

// The window system's update loop
fn run(context: &mut Context, _query: ComponentQuery) {
    let read = context.read();
    let pipeline = read.pipeline.read();
    if read.input.map_changed("toggle_fullscreen") {
        pipeline.window.set_fullscreen(read.input.map_toggled("toggle_fullscreen"));
    }
    if read.input.map_changed("toggle_vsync") {
        pipeline.window.set_vsync(read.input.map_toggled("toggle_vsync"));
    }
}

// Create a system that'll allow us to disable/enable fullscreen and vsync
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).build();
    write.input.bind_key_toggle(Keys::F5, "toggle_fullscreen");
    write.input.bind_key_toggle(Keys::F6, "toggle_vsync");
}
