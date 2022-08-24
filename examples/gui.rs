use cflake_engine::prelude::*;

// Create an app that will render some GUI text
fn main() {
    App::default()
        .set_window_title("cflake engine GUI example")
        .insert_update(update)
        .execute();
}

// Update the UI and render some cool text
fn update(world: &mut World) {
    let mut ui = world.get_mut::<UserInterface>().unwrap();
    let time = world.get::<Time>().unwrap();
    let ctx = ui.as_mut().as_mut();
    egui::Window::new("Test window").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Delta (s/f): ");
            ui.label(time.delta_f32().to_string());
        });

        ui.horizontal(|ui| {
            ui.label("FPS (f/s): ");
            ui.label((1.0 / time.delta_f32()).to_string());
        });
    });
}
