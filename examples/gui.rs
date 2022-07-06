use cflake_engine::prelude::*;

// Create an app that will render some GUI text 
fn main() {
    App::default()
        .set_window_title("cflake engine GUI example")
        .insert_system(system)
        .execute();
}

// This is an init event that will be called at the start of the game
fn init(world: &mut World) {
    let ui = world.get_mut::<&mut UserInterface>().unwrap();
    let ctx = ui.as_mut();
    let mut style = (*ctx.style()).clone();
    use cflake_engine::gui::egui::*;
    use FontFamily::Proportional;
    use TextStyle::*;
    style.text_styles = [
        (Heading, FontId::new(30.0, Proportional)),
        (Body, FontId::new(18.0, Proportional)),
        (Monospace, FontId::new(14.0, Proportional)),
        (Button, FontId::new(14.0, Proportional)),
        (Small, FontId::new(10.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}

// Update the UI and render some cool text
fn update(world: &mut World) {
    let (ui, time) = world.get_mut::<(&mut UserInterface, &mut Time)>().unwrap();
    let ctx = ui.as_mut();
    egui::Window::new("Test window").show(&ctx, |ui| {
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

// This is an example system that will register specific events
fn system(events: &mut Events) {
    events.registry::<Init>().insert(init);
    events.registry::<Update>().insert(update);
}
