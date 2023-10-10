use app::prelude::*;

fn main() {
    App::new()
        .insert_init(init)
        .insert_update(update)
        .execute().unwrap();
}

fn init(world: &mut World, _: &Init) {
    let mut input = world.get_mut::<Input>().unwrap();
    input.bind_button("w", KeyboardButton::KeyW);
}

fn update(world: &mut World, _: &Update) {
    let mut input = world.get::<Input>().unwrap();

    if (input.get_button("w").pressed()) {
        println!("Hello World!")
    }
}