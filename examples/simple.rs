use cflake_engine::prelude::{vek::Lerp, *};

const ASSETS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/");

// Create a game that will draw a simple mesh onto the screen and a movable camera
fn main() {
    App::default()
        .set_window_title("cflake engine simple example")
        .set_user_assets_folder_path(ASSETS_PATH)
        .insert_init(init)
        .execute();
}

// This is an init event that will be called at the start of the game
fn init(world: &mut World) {
    // Get the graphics resources
    let mut ctx = world.get_mut::<Context>().unwrap();
    let mut standard_materials = world.get_mut::<Storage<Standard>>().unwrap();
    let mut sky_materials = world.get_mut::<Storage<Sky>>().unwrap();
    let mut albedo_maps = world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let mut normal_maps = world.get_mut::<Storage<NormalMap>>().unwrap();
    let mut mask_maps = world.get_mut::<Storage<MaskMap>>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();

    // Create a perspective camera and insert it into the world as an entity (and update the scene settings)
    let camera = Camera::new(70.0, 0.003, 6000.0, 16.0 / 9.0);
    ecs.insert((
        camera,
        Location::at_xyz(0.0, 1.5, 4.0),
        Rotation::rotation_x(-20.0f32.to_radians()),
    ));

    // Create a directional light insert it as a light entity (and update the scene settings)
    let light = DirectionalLight {
        color: vek::Rgb::new(255, 255, 230),
        strength: 16.0,
    };
    ecs.insert((light, Rotation::rotation_x(90f32.to_radians())));

    // Create the missing albedo map texture
    let albedo_map = assets
        .load_with::<AlbedoMap>(
            "user/textures/metal/diffuse.jpg",
            (&mut ctx, TextureImportSettings::default()),
        )
        .unwrap();
    let albedo_map = albedo_maps.insert(albedo_map);

    // Create the default normal map texture
    let normal_map = assets
        .load_with::<NormalMap>(
            "user/textures/metal/normal.jpg",
            (&mut ctx, TextureImportSettings::default()),
        )
        .unwrap();
    let normal_map = normal_maps.insert(normal_map);

    // Create the default mask map texture
    let mask_map = assets
        .load_with::<MaskMap>(
            "user/textures/metal/mask.jpg",
            (&mut ctx, TextureImportSettings::default()),
        )
        .unwrap();
    let mask_map = mask_maps.insert(mask_map);

    // Create the default cube primitive mesh
    let cube = assets.load_with::<Mesh>(
        "engine/meshes/cube.obj",
        (
            &mut ctx,
            MeshImportSettings::default(),
        ),
    )
    .unwrap();
    let cube = meshes.insert(cube);

    // Create a new material instance with the normal map texture
    let material = standard_materials.insert(Standard {
        albedo_map,
        normal_map,
        mask_map,
        bumpiness: 1.4,
        roughness: 1.0,
        ambient_occlusion: 1.0,
        metallic: 1.0,
        scale: vek::Vec2::one(),
        tint: vek::Rgb::white(),
    });

    // Create a new material surface for rendering
    let pipeid = ctx.get_pipe_id::<SpecializedPipeline<Standard>>().unwrap();
    let surface = Surface::new(cube, material, pipeid);

    // Insert a new entity that contains the valid surface
    ecs.insert((surface, Renderer::default()));

    // Load in the texture
    let texture = albedo_maps.insert(
        assets
            .load_with::<AlbedoMap>(
                "engine/textures/sky_gradient.png",
                (&mut ctx, TextureImportSettings::default()),
            )
            .unwrap(),
    );

    // Create the default sky material
    let material = Sky {
        gradient: texture,
        sun_intensity: 15.0,
        sun_size: 1.05,
        cloud_coverage: 0.0,
        cloud_speed: 0.0,
    };

    // Create the default Sky material pipeline and default Sky sphere surface
    let material = sky_materials.insert(material);
    let pipeid = ctx.get_pipe_id::<SpecializedPipeline<Sky>>().unwrap();
    let renderer = Renderer::default();
    let sphere = assets
        .load_with::<Mesh>(
            "engine/meshes/sphere.obj",
            (
                &mut ctx,
                MeshImportSettings {
                    invert_triangle_ordering: true,
                    use_tangents: false,
                    use_normals: false,
                    ..Default::default()
                },
            ),
        )
        .unwrap();
    let sphere = meshes.insert(sphere);
    let surface = Surface::new(sphere, material, pipeid);

    // Insert it as a new entity
    ecs.insert((renderer, surface, Scale::from(vek::Vec3::one() * 5000.0)));
}