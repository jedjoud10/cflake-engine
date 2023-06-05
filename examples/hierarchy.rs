
use cflake_engine::prelude::*;

// Mesh example game window
fn main() {
    App::default()
        .set_app_name("cflake engine physics example")
        .insert_init(init)
        .insert_update(update)
        .set_window_fullscreen(true)
        .execute();
}

// Creates a movable camera, and sky entity
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut pbrs = world.get_mut::<Storage<PbrMaterial>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let pipelines = world.get::<Pipelines>().unwrap();
    let id = pipelines.get::<PbrMaterial>().unwrap();
    let renderer = world.get::<DeferredRenderer>().unwrap();
    let cube = renderer.cube.clone();

    let material = pbrs.insert(PbrMaterial {
        albedo_map: None,
        normal_map: None,
        mask_map: None,
        bumpiness_factor: 0.9,
        roughness_factor: 1.0,
        metallic_factor: 1.0,
        ambient_occlusion_factor: 4.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one(),
    });

    let surface = Surface::new(cube.clone(), material.clone(), id.clone());
    let renderer = Renderer::default();
    let parent = scene.insert((
        surface,
        renderer,
        Position::default(),
        Rotation::default(),
    ));
    
    let surface = Surface::new(cube.clone(), material.clone(), id.clone());
    let renderer = Renderer::default();
    let child = scene.insert((
        surface,
        renderer,
        RelativePosition::at_y(5.0),
        RelativeRotation::default(),
        Position::default(),
        Rotation::default(),
    ));
    scene.attach(child, parent).unwrap();


    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera::default(),
        CameraController::default(),
    ));

    let light = DirectionalLight {
        color: vek::Rgb::one() * 4.6,
    };
    let rotation = vek::Quaternion::rotation_x(-45.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));
}

fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut time = world.get::<Time>().unwrap();
    let (parent, rotation) = scene.find_mut::<(&Parent, &mut Rotation)>().unwrap();
    rotation.rotate_x(-0.3 * time.delta().as_secs_f32());

    let (child, rotation) = scene.find_mut::<(&Child, &mut RelativeRotation)>().unwrap();
    //rotation.rotate_x(-0.3 * time.delta().as_secs_f32());
}