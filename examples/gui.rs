use cflake_engine::prelude::*;

// Create an app that will render some GUI text
fn main() {
    App::default()
        .set_window_title("cflake engine GUI example")
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// This is an init event that will be called at the start of the game
fn init(world: &mut World) {
    let mut ui = world.get_mut::<UserInterface>().unwrap();
    let ctx = ui.as_mut().as_mut();
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