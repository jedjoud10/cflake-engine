
use cflake_engine::prelude::*;

// Mesh example game window
fn main() {
    App::default()
        .set_app_name("cflake engine physics example")
        .insert_init(init)
        .execute();
}
// Creates a movable camera, and sky entity
fn init(world: &mut World) {
    // Fetch the required resources from the world
    let assets = world.get::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut pbrs = world.get_mut::<Storage<PbrMaterial>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let pipelines = world.get::<Pipelines>().unwrap();
    let id = pipelines.get::<PbrMaterial>().unwrap();

    // Get the default meshes from the forward renderer
    let renderer = world.get::<ForwardRenderer>().unwrap();
    let plane = renderer.plane.clone();
    let sphere = renderer.sphere.clone();

    // Create a new material instance
    let material = pbrs.insert(PbrMaterial {
        albedo_map: None,
        normal_map: None,
        mask_map: None,
        bumpiness_factor: 1.0,
        roughness_factor: 1.0,
        metallic_factor: 1.0,
        ambient_occlusion_factor: 3.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one(),
    });

    // Create a simple floor and add the entity
    let surface = Surface::new(plane, material.clone(), id.clone());
    let renderer = Renderer::default();
    let scale = Scale::uniform(25.0);
    scene.insert((surface, renderer, scale));

    // Create a prefab that contains the renderer, customized surface, and default position
    let renderer = Renderer::default();
    let position = Position::default();
    let rotation = Rotation::default();
    let surface = Surface::new(sphere, material, id);
    let rigidbody = RigidBody::new(RigidBodyType::Dynamic);
    scene.prefabify("sphere", (renderer, position, rotation, surface, rigidbody));

    // ADD THE ENTITIES NOW!!
    for x in 0..25 {
        let mut entry = scene.instantiate("sphere").unwrap();
        let position = entry.get_mut::<Position>().unwrap();
        *position = Position::at_xyz((x / 5) as f32 * 4.0, 5.0, (x % 5) as f32 * 4.0);
    }

    // Create a movable camera
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera::default(),
        CameraController::default(),
    ));

    // Create a directional light
    let light = DirectionalLight {
        color: vek::Rgb::one() * 3.6,
    };
    let rotation = vek::Quaternion::rotation_x(-15.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));
}
