use ecs::component::defaults::*;
use ecs::component::*;

use main::core::{Context, WriteContext};
use main::ecs;
use main::input::Keys;
use main::input::MapType::Toggle;
use main::rendering::object::PipelineTask;
use main::rendering::pipeline::pipec;

// The window system's update loop
fn run(context: Context, query: ComponentQuery) {
    let read = context.read();
    let pipeline = read.pipeline.read().unwrap();
    if read.input.map_toggle_changed("toggle_fullscreen") { pipeline.window.set_fullscreen(read.input.map_toggled("toggle_fullscreen")); dbg!() }
    if read.input.map_toggle_changed("toggle_vsync") { pipeline.window.set_vsync(read.input.map_toggled("toggle_vsync")); dbg!() }
}

// Create a system that'll allow us to disable/enable fullscreen and vsync
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .build();
    write.input.bind_key(Keys::F5, "toggle_fullscreen", Toggle);
    write.input.bind_key(Keys::F6, "toggle_vsync", Toggle);
}
