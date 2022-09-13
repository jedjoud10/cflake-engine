use cflake_engine::prelude::{vek::Lerp, *};

const ASSETS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/");
const SENSIVITY: f32 = 0.0007;
const SPEED: f32 = 10.0;
const ROTATION_SMOOTH_SPEED: f32 = 80.0;
const VELOCITY_SMOOTH_SPEED: f32 = 80.0;

// Create a game that will draw a simple mesh onto the screen and a movable camera
fn main() {
    App::default()
        .set_window_title("cflake engine mesh example")
        .set_user_assets_folder_path(ASSETS_PATH)
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// This is an init event that will be called at the start of the game
fn init(world: &mut World) {
    // Get the graphics resources
    let mut ctx = world.get_mut::<Context>().unwrap();
    let _shading = world.get_mut::<ClusteredShading>().unwrap();
    let mut standard_materials = world.get_mut::<Storage<Standard>>().unwrap();
    let mut sky_materials = world.get_mut::<Storage<Sky>>().unwrap();
    let mut albedo_maps = world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let mut normal_maps = world.get_mut::<Storage<NormalMap>>().unwrap();
    let mut mask_maps = world.get_mut::<Storage<MaskMap>>().unwrap();
    let _shaders = world.get_mut::<Storage<Shader>>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();

    // Get the other resources
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let mut keyboard = world.get_mut::<Keyboard>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();

    // Create a perspective camera and insert it into the world as an entity (and update the scene settings)
    let camera = Camera::new(90.0, 0.3, 10000.0, 16.0 / 9.0);
    let _camera = ecs.insert((
        camera,
        Location::at_z(5.0),
        Rotation::default(),
        Velocity::default(),
    ));

    // We will also register some new keybinds for the camera controller
    keyboard.bind("forward", Key::W);
    keyboard.bind("backward", Key::S);
    keyboard.bind("left", Key::A);
    keyboard.bind("right", Key::D);

    // Create a directional light insert it as a light entity (and update the scene settings)
    let light = DirectionalLight {
        color: vek::Rgb::broadcast(255),
        strength: 12.0,
    };
    ecs.insert((light, Rotation::rotation_x(45f32.to_radians())));

    // Create the default albedo map texture
    let albedo_map = assets
        .load_with::<AlbedoMap>(
            "engine/textures/missing.png",
            (&mut ctx, TextureImportSettings::default()),
        )
        .unwrap();
    let albedo_map = albedo_maps.insert(albedo_map);

    // Create the default normal map texture
    let normal_map = assets
        .load_with::<NormalMap>(
            "engine/textures/bumps.png",
            (&mut ctx, TextureImportSettings::default()),
        )
        .unwrap();
    let normal_map = normal_maps.insert(normal_map);

    // Create the default mask map texture
    let mask_map = assets
        .load_with::<MaskMap>(
            "engine/textures/mask.png",
            (&mut ctx, TextureImportSettings::default()),
        )
        .unwrap();
    let mask_map = mask_maps.insert(mask_map);
    
    // Create the default cube primitive mesh
    let cube =  meshes.insert(
        assets
            .load_with::<Mesh>(
                "engine/meshes/cube.obj",
                (&mut ctx, MeshImportSettings::default()),
            ).unwrap(),
    );

    // Create a new material instance with the normal map texture
    let material = standard_materials.insert(Standard {
        albedo_map,
        normal_map,
        mask_map,
        bumpiness: 0.2,
        roughness: 1.0,
        ambient_occlusion: 1.0,
        metallic: 1.0,
        scale: vek::Vec2::broadcast(3.0),
        tint: vek::Rgb::white(),
    });

    // Create a new material surface for rendering
    let pipeid = ctx.get_pipe_id::<SpecializedPipeline<Standard>>().unwrap();
    let surface = Surface::new(cube.clone(), material.clone(), pipeid.clone());
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

#[derive(Component, Default)]
struct Velocity {
    velocity: vek::Vec3<f32>,
}
// We will use this update event to move the camera around
fn update(world: &mut World) {
    let mut postprocessing = world.get_mut::<PostProcessing>().unwrap();
    let mut ui = world.get_mut::<UserInterface>().unwrap();
    let ctx = ui.as_mut().as_mut();
    egui::Window::new("Test window").show(ctx, |ui| {
        ui.add(egui::DragValue::new(&mut postprocessing.exposure).speed(0.1)); // 02
        ui.add(egui::Slider::new(
            &mut postprocessing.tonemapping_strength,
            0.0f32..=1.0,
        ));
        ui.add(egui::DragValue::new(&mut postprocessing.gamma).speed(0.1));
    });

    let shading = world.get::<ClusteredShading>().unwrap();
    let window = world.get_mut::<Window>().unwrap();

    // Get the input resources
    let keyboard = world.get::<Keyboard>().unwrap();
    let mouse = world.get::<Mouse>().unwrap();

    if keyboard.held(Key::H) {
        window.raw().set_cursor_grab(false);
        window.raw().set_cursor_visible(true);
        return;
    } else {
        window.raw().set_cursor_grab(true);
        window.raw().set_cursor_visible(false);
    }

    // Get the other resources
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let time = world.get::<Time>().unwrap();

    if let Some(mut entry) = shading.main_camera().and_then(|c| ecs.entry_mut(c)) {
        // Get the location and rotation since we will update them
        let (location, rotation, velocity) = entry
            .as_query::<(&mut Location, &mut Rotation, &mut Velocity)>()
            .unwrap();

        // Forward and right vectors relative to the camera
        let forward = rotation.forward();
        let right = rotation.right();
        let mut temp = vek::Vec3::<f32>::default();

        // Update the velocity in the forward and backward directions
        if keyboard.held("forward") {
            temp += forward;
        } else if keyboard.held("backward") {
            temp += -forward;
        }

        // Update the velocity in the left and right directions
        if keyboard.held("left") {
            temp += -right;
        } else if keyboard.held("right") {
            temp += right;
        }

        velocity.velocity = vek::Vec3::lerp_unclamped_precise(
            velocity.velocity,
            temp,
            time.delta_f32() * VELOCITY_SMOOTH_SPEED,
        );

        // Update the location with the new velocity
        **location += velocity.velocity * time.delta_f32() * SPEED;

        // Calculate a new rotation and apply it
        let pos = mouse.position();
        let rot = vek::Quaternion::rotation_y(-pos.x as f32 * SENSIVITY)
            * vek::Quaternion::rotation_x(-pos.y as f32 * SENSIVITY);
        **rotation = vek::Quaternion::lerp_unclamped_precise(
            **rotation,
            rot,
            time.delta_f32() * ROTATION_SMOOTH_SPEED,
        );
    }
}
