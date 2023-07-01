
use cflake_engine::prelude::*;

// Hierarchy example game window
fn main() {
    App::default()
        .set_app_name("cflake engine hierarchy example")
        .insert_init(init)
        .insert_update(update)
        .set_window_fullscreen(true)
        .execute();
}

// Creates a movable camera and a few interconnected entities
fn init(world: &mut World) {
    let mut pbrs = world.get_mut::<Storage<PbrMaterial>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let pipelines = world.get::<Pipelines>().unwrap();
    let id = pipelines.get::<PbrMaterial>().unwrap();
    let renderer = world.get::<DeferredRenderer>().unwrap();
    let cube = renderer.cube.clone();

    // Create a default material
    let material = pbrs.insert(PbrMaterial {
        albedo_map: None,
        normal_map: None,
        mask_map: None,
        bumpiness_factor: 0.9,
        roughness_factor: 1.0,
        metallic_factor: 0.0,
        ambient_occlusion_factor: 0.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one(),
    });

    // Create a parent entity that will contain multiple children
    let surface = Surface::new(cube.clone(), material.clone(), id.clone());
    let renderer = Renderer::default();
    let parent = scene.insert((
        surface,
        renderer,
        Position::default(),
        Rotation::default(),
    ));
    
    // Create a child entity that we will attach to the parent entity
    let surface = Surface::new(cube.clone(), material.clone(), id.clone());
    let renderer = Renderer::default();
    let child1 = scene.insert((
        surface,
        renderer,
        LocalPosition::at_y(5.0),
        LocalRotation::default(),
        Position::default(),
        Rotation::default(),
    ));
    scene.attach(child1, parent).unwrap();

    // Create another child entity that we will attach to the other child
    let surface = Surface::new(cube.clone(), material.clone(), id.clone());
    let renderer = Renderer::default();
    let child2 = scene.insert((
        surface,
        renderer,
        LocalPosition::at_y(10.0),
        LocalRotation::default(),
        Position::default(),
        Rotation::default(),
    ));
    scene.attach(child2, child1).unwrap();

    // Create the main camera
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera::default(),
        CameraController::default(),
    ));

    // Create the main light source
    let light = DirectionalLight::default();
    let rotation = vek::Quaternion::rotation_x(-45.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));
}

// Update the local and global rotations of the children and parent
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut time = world.get::<Time>().unwrap();

    let (_, pos) = scene.find::<(&Child, &Position)>().unwrap();

    for (_, rotation, relative_rotation) in scene.query_mut::<(&Parent, &mut Rotation, Option<&mut LocalRotation>)>() {
        if let Some(relative_rotation) = relative_rotation {
            //relative_rotation.rotate_x(-0.3 * time.delta().as_secs_f32());
        } else {
            rotation.rotate_x(0.15 * time.delta().as_secs_f32());
        }
    }
}