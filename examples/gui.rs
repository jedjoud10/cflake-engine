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
    let _time = world.get::<Time>().unwrap();

    egui::Window::new("Test window").show(&ui, |ui| {
        ui.label("Catgirl enjoyer idk test text");
        ui.label("ABCDabcd1234");
    });
}
