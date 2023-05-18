use cflake_engine::prelude::*;

fn main() {
    App::default()
        .set_logging_level(LevelFilter::Off)
        .insert_init(init)
        .execute();
}

struct TestResource {
    a: f32,
    b: i32,
}

fn init(world: &mut World) {
    // asset loader
    let mut loader = world.get_mut::<Assets>().unwrap();
    
    // graphics context or API
    let graphics = world.get::<Graphics>().unwrap();

    // scene
    let mut scene = world.get_mut::<Scene>().unwrap();

    // loads the diffuse texture
    asset!(loader, "user/textures/diffuse2.jpg", "/examples/assets/");

    let albedo = 
        loader.load::<AlbedoMap>(("user/textures/diffuse2.jpg", graphics.clone()))
        .unwrap();

    // Create a movable camera
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera::default(),
        CameraController::default(),
    ));
}