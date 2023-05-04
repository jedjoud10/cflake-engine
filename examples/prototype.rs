use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
}

// Camera controller update executed every tick
fn update(world: &mut World) {
    let gui = world.get_mut::<Interface>().unwrap();
 
    egui::Window::new("Prototyping").show(&gui, |ui| {
    });
}
