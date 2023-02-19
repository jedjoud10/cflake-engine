use std::time::Instant;

use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_init(init)
        .insert_update(update)
        .set_frame_rate_limit(FrameRateLimit::Unlimited)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let assets = world.get::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut pipelines = world.get_mut::<Pipelines>().unwrap();
    pipelines.register::<Basic>(&graphics, &assets).unwrap();

    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut materials = world.get_mut::<Storage<Basic>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let id = pipelines.get::<Basic>().unwrap();

    let material = materials.insert(Basic {
        albedo_map: None,
        normal_map: None,
        roughness: 0.0,
        tint: vek::Rgb::default(),
    });

    let mesh = assets
        .load::<Mesh>(("engine/meshes/cube.obj", &*graphics))
        .unwrap();
    let vertices = mesh.vertices();
    let positions =
        vertices.attribute::<attributes::Position>().unwrap();
    let mesh = meshes.insert(mesh);

    let surface = Surface::new(mesh, material, id);
    let renderer = Renderer::new(true, vek::Mat4::default());
    scene.insert((surface, renderer));
}

fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let input = world.get::<Input>().unwrap();

    if input.get_button(Button::F5).pressed() {
        dbg!(time.average_fps());
    }
}
