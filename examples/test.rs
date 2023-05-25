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
    let mut scene = world.get_mut::<Scene>().unwrap();

    // create camera entity
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera::default(),
        CameraController::default(),
    ));

    // create sun source light
    scene.insert((
        DirectionalLight {
            color: vek::Rgb::one(),
        },
        Rotation::rotation_x(-15.0f32.to_radians()),
    ));

    // fetch resources from world
    let mut assets = world.get_mut::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let _meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut pbrs = world.get_mut::<Storage<PbrMaterial>>().unwrap();
    let mut pipelines = world.get_mut::<Pipelines>().unwrap();
    let forward_renderer = world.get::<ForwardRenderer>().unwrap();

    // register the PBR material to use it
    let id = pipelines
        .register::<PbrMaterial>(&graphics, &mut assets)
        .unwrap();

    // load a sphere mesh
    let sphere: Handle<Mesh> = forward_renderer.sphere.clone();

    // create a PBR material
    let material = pbrs.insert(PbrMaterial {
        albedo_map: None,
        normal_map: None,
        mask_map: None,
        bumpiness: 1.0,
        roughness: 1.0,
        metallic: 1.0,
        ambient_occlusion: 3.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one(),
    });

    let surface = Surface::new(sphere, material, id);
    scene.insert((surface, Renderer::default()));
}
