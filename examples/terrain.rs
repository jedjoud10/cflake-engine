use cflake_engine::prelude::*;

// Terrain example game window
fn main() {
    App::default()
        .set_app_name("cflake engine terrain example")
        .set_window_fullscreen(true)
        .set_logging_level(LevelFilter::Error)
        .insert_init(init)
        .insert_update(update)
        .execute();
}

/*
// Voxel graph that we will generate
fn graph(x: VoxelNode<f32>, y: VoxelNode<f32>, z: VoxelNode<f32>) -> VoxelNode<f32> {
    use cflake_engine::terrain::graph::*;
    let added: VoxelNode<f32> = x + y - y;
}
*/

// Creates a movable camera, sky entity, and procedural terrain
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let assets = world.get::<Assets>().unwrap();

    // Setup the assets that will be loaded
    asset!(assets, "user/textures/diffuse1.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal1.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask1.jpg", "/examples/assets/");
    asset!(assets, "user/textures/diffuse2.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal2.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask2.jpg", "/examples/assets/");
    asset!(assets, "user/textures/diffuse3.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal3.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask3.jpg", "/examples/assets/");

    // Create the terrain sub material settings
    let settings = TerrainSubMaterialsSettings {
        materials: [
            TerrainSubMaterial {
                diffuse: "user/textures/diffuse1.jpg".to_string(),
                normal: "user/textures/normal1.jpg".to_string(),
                mask: "user/textures/mask1.jpg".to_string(),
            },
            TerrainSubMaterial {
                diffuse: "user/textures/diffuse2.jpg".to_string(),
                normal: "user/textures/normal2.jpg".to_string(),
                mask: "user/textures/mask2.jpg".to_string(),
            },
            TerrainSubMaterial {
                diffuse: "user/textures/diffuse3.jpg".to_string(),
                normal: "user/textures/normal3.jpg".to_string(),
                mask: "user/textures/mask3.jpg".to_string(),
            },
        ].to_vec(),
        scale: TextureScale::default(),
        sampler: SamplerSettings {
            mipmaps: SamplerMipMaps::Auto,
            ..Default::default()
        },
    };

    // Create the terrain generator's settings
    let settings = TerrainSettings::new(
        &graphics,
        128,
        false,
        true,
        false,
        8,
        1024,
        6,
        1.0,
        Some(settings),
    )
    .unwrap();

    // Drop (needed) to insert settings
    drop(graphics);
    drop(assets);
    world.insert(settings);

    // Create a movable camera
    let mut scene = world.get_mut::<Scene>().unwrap();
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera {
            near: 0.3,
            far: 9000.0,
            ..Default::default()
        },
        ChunkViewer::default(),
        CameraController::default(),
    ));

    // Create a directional light
    let light = DirectionalLight::default();
    let rotation = vek::Quaternion::rotation_x(-20.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));

    let mut pbrs = world.get_mut::<Storage<PbrMaterial>>().unwrap();

    // Create a new material instance
    let material = pbrs.insert(PbrMaterial {
        albedo_map: None,
        normal_map: None,
        mask_map: None,
        bumpiness_factor: 1.0,
        roughness_factor: 1.0,
        metallic_factor: 0.0,
        ambient_occlusion_factor: 1.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one(),
    });

    let pipelines = world.get::<Pipelines>().unwrap();
    let id = pipelines.get::<PbrMaterial>().unwrap();
    let renderer = world.get::<DeferredRenderer>().unwrap();
    let sphere = renderer.sphere.clone();
    let renderer = Renderer::default();
    let position = Position::default();
    let surface = Surface::new(sphere, material, id);
    scene.prefabify("sphere", (renderer, position, surface));
}

// Updates the light direction and quites from the engine
fn update(world: &mut World) {
    let mut state = world.get_mut::<State>().unwrap();
    let input = world.get::<Input>().unwrap();
    let time = world.get::<Time>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Rotation the light
    /*
    if let Some((rotation, light)) = scene.find_mut::<(&mut Rotation, &mut DirectionalLight)>() {
        let value = (time.elapsed().as_secs_f32() * 0.1).sin();
        **rotation = Quaternion::rotation_x((value * 90.0 - 90.0).to_radians());
        let noon = vek::Rgb::new(255.0f32, 231.0, 204.0);
        let sunrise = vek::Rgb::new(255.0f32, 151.0, 33.0);
        let interpolated = vek::Lerp::lerp(noon, sunrise, value.abs());
        light.color = interpolated.map(|x| x as u8);
    }
    */

    // Exit the game when the user pressed Escape
    if input.get_button(KeyboardButton::Escape).pressed() {
        *state = State::Stopped;
    }

    // Create a new sphere in front of the camera when we press the right mouse button
    if input.get_button(MouseButton::Right).pressed() {
        let (_, position, rotation) = scene.find::<(&Camera, &Position, &Rotation)>().unwrap();
        let mut entry = scene.instantiate("sphere").unwrap();
        **entry.get_mut::<Position>().unwrap() = rotation.forward() * 3.0 + **position;
    }
}
