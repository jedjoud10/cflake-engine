use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_window_title("cflake engine prototype 2 example")
        .insert_init(init)
        .insert_update(update)
        .execute();
}

#[derive(Component)]
struct Temp;

fn init(world: &mut World) {
    let mut ecs = world.get_mut::<EcsManager>().unwrap();
    ecs.insert((Transform::default(), Temp));
    ecs.insert(Temp);
}

fn update(world: &mut World) {
    let ecs = world.get::<EcsManager>().unwrap();

    for (transform, temp) in ecs.view::<(Option<&Transform>, Option<&Temp>)>().unwrap() {
        println!("temp: {}", temp.is_some());
        println!("transform: {}", transform.is_some());
    }
}