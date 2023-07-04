use cflake_engine::prelude::*;

// Terrain example game window
fn main() {
    App::default()
        .set_app_name("cflake engine terrain example")
        .insert_init(init)
        .insert_update(update)
        .set_window_fullscreen(true)
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

    // Terrain mesher/octree settings
    let mesher = TerrainMeshSettings {
        size: 64,
        collisions: true,
        max_octree_depth: 9,
        quality: 1.0,
    };

    // Terrain memory settings
    let memory = TerrainMemorySettings {
        allocation_count: 4,
        sub_allocation_count: 1024,
    };

    // Terrain rendering settings
    let rendering = TerrainRenderingSettings {
        blocky: true,
        flat_normals: false,
        derived_normals: true,
        flat_colors: true,
        submaterials: Some(TerrainSubMaterialsSettings {
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
        }),
    };

    // Create the terrain generator's settings
    let settings = TerrainSettings::new(
        mesher, memory, rendering
    ).unwrap();

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
            far: 18000.0,
            ..Default::default()
        },
        ChunkViewer::default(),
        CameraController::default(),
    ));

    // Create a directional light
    let light = DirectionalLight { intensity: 1.5, color: vek::Rgb::broadcast(255)  };
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

    // Create a prefab that contains the cube entity and it's components
    let surface = Surface::new(renderer.cube.clone(), material, id);
    let renderer = Renderer::default();
    let position = Position::default();
    let rotation = Rotation::default();
    let rigidbody = RigidBodyBuilder::new(RigidBodyType::Dynamic).build();
    let velocity = Velocity::default();
    let angular_velocity = AngularVelocity::default();
    let collider = CuboidColliderBuilder::new(10.0, vek::Extent3::broadcast(1.0)).build();
    scene.prefabify("cube", (renderer, position, rotation, surface, rigidbody, collider, velocity, angular_velocity));
}

// Updates the light direction and quites from the engine
fn update(world: &mut World) {
    let mut state = world.get_mut::<State>().unwrap();
    let input = world.get::<Input>().unwrap();
    let time = world.get::<Time>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Rotation the light
    if let Some((rotation, light)) = scene.find_mut::<(&mut Rotation, &mut DirectionalLight)>() {
        let value = (time.elapsed().as_secs_f32() * 0.1).sin();
        **rotation = Quaternion::rotation_x((value * 85.0 - 90.0).to_radians());
        let noon = vek::Rgb::new(255.0f32, 250.0, 240.0);
        let sunrise = vek::Rgb::new(255.0f32, 179.0, 92.0);
        let interpolated = vek::Lerp::lerp(noon, sunrise, value.abs().powf(5.0));
        light.color = interpolated.map(|x| x as u8);
    }

    // Exit the game when the user pressed Escape
    if input.get_button(KeyboardButton::Escape).pressed() {
        *state = State::Stopped;
    }

    // Create a new cube in front of the camera when we press the right mouse button
    if input.get_button(MouseButton::Right).pressed() {
        let (_, position, rotation) = scene.find::<(&Camera, &Position, &Rotation)>().unwrap();
        let mut entry = scene.instantiate("cube").unwrap();
        **entry.get_mut::<Position>().unwrap() = rotation.forward() * 3.0 + **position;
    }
}
