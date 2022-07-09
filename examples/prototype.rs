use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .insert_update(update)
        .set_window_title("cflake engine prototype example")    
        .execute();
}

fn update(world: &mut World) {
    let (submeshes, scene, ctx) = world.get_mut::<(&Storage<Mesh>, &SceneSettings, &mut Context)>().unwrap();
    let cube = submeshes.get(&scene.cube());
    let buffer = cube.attributes().get::<Position>().unwrap();
    println!("{}", buffer.len());
}