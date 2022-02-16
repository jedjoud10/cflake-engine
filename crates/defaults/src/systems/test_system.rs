use main::core::World;
use main::ecs::event::EventKey;
use main::gui::egui;

// A simple system that we can use for testing
fn run(_world: &mut World, _data: EventKey) {
    // GUI moment
    let gui = &_world.gui.egui;
    egui::Window::new("Egui with Winit and OpenGL")
        .vscroll(false)
        .hscroll(false)
        .collapsible(false)
        .fixed_pos(egui::pos2(0.0, 0.0))
        .resizable(false)
        .show(&gui, |ui| {
            ui.separator();
            ui.label("Anime girls.");
            ui.label(" ");
            if ui.button("Le button").clicked() {}
        });
}

// Create the system
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().with_run_event(run).build();
}
