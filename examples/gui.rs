use cflake_engine::*;

// An empty game window
fn main() {
    cflake_engine::start("cflake-examples/gui", init)
}

// Init the empty world
fn init(world: &mut World) {
    world.events.insert(run);
}

// Simple GUI
fn run(world: &mut World) {
    gui::egui::Window::new("Debug Window")
        .vscroll(false)
        .hscroll(false)
        .resizable(false)
        .show(&mut world.gui.egui, |ui| {
            // Timings
            ui.separator();
            ui.heading("Timings");
            ui.label(format!("Time: {:.1}", world.time.elapsed()));
            ui.label(format!("Delta: {:.3}", world.time.average_delta()));
            ui.label(format!("FPS: {:.1}", 1.0 / world.time.average_delta()));
        });
}
