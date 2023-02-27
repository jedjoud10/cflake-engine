use cflake_engine::prelude::*;

// Create an app that will render some GUI text
fn main() {
    App::default()
        .set_app_name("cflake engine GUI example")
        .insert_update(update)
        .execute();
}

// Update the UI and render some cool text
fn update(world: &mut World) {
    let ui = world.get_mut::<Interface>().unwrap();
    let time = world.get::<Time>().unwrap();
    
    egui::Window::new("Test window").show(&ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Delta (s/f): ");
            ui.label(time.delta().as_secs_f32().to_string());
        });

        ui.horizontal(|ui| {
            ui.label("FPS (f/s): ");
            ui.label((1.0 / time.delta().as_secs_f32()).to_string());
        });
    });
}