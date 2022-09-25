use cflake_engine::prelude::{*};

const ASSETS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/");
const SENSIVITY: f32 = 0.0007;
const SPEED: f32 = 10.0;

// Create a game that will draw a simple mesh onto the screen and a movable camera
fn main() {
    App::default()
        .set_window_title("cflake engine mesh example")
        .set_user_assets_folder_path(ASSETS_PATH)
        .insert_init(init)
        .set_window_fullscreen(true)
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
    let mut input = world.get_mut::<Input>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();

    // Create a perspective camera and insert it into the world as an entity (and update the scene settings)
    let camera = Camera::new(90.0, 0.3, 1000.0, 16.0 / 9.0);
    let _camera = ecs.insert((camera, Location::at_z(5.0), Rotation::default()));

    asset!(&mut assets, "assets/user/diffuse.png");
    asset!(&mut assets, "assets/user/normal.png");
    asset!(&mut assets, "assets/user/mask.png");

    // We will also register some new keybinds for the camera controller
    input.bind_key("forward", Key::W);
    input.bind_key("backward", Key::S);
    input.bind_key("left", Key::A);
    input.bind_key("right", Key::D);
    input.bind_axis("x rotation", Axis::MousePositionX);
    input.bind_axis("y rotation", Axis::MousePositionY);
    

    // Create a directional light insert it as a light entity (and update the scene settings)
    let light = DirectionalLight {
        color: vek::Rgb::new(255, 243, 196),
        strength: 10.0,
    };

    let b1 = Rotation::rotation_x(45f32.to_radians());
    let b2 = Rotation::rotation_z(45f32.to_radians());
    ecs.insert((light, b2 * b1));

    // Load the albedo map texture
    let albedo_map = assets
        .load_with::<AlbedoMap>(
            "user/diffuse.png",
            (&mut ctx, TextureImportSettings::default()),
        )
        .unwrap();
    let albedo_map = albedo_maps.insert(albedo_map);

    // Load the normal map texture
    let normal_map = assets
        .load_with::<NormalMap>(
            "user/normal.png",
            (&mut ctx, TextureImportSettings::default()),
        )
        .unwrap();
    let normal_map = normal_maps.insert(normal_map);
    
    // Load the mask map texture
    let mask_map = assets
        .load_with::<MaskMap>(
            "user/mask.png",
            (&mut ctx, TextureImportSettings::default()),
        )
        .unwrap();
    let mask_map = mask_maps.insert(mask_map);

    // Create the default cube primitive mesh
    let cube = meshes.insert(
        assets
            .load_with::<Mesh>(
                "engine/meshes/cube.obj",
                (&mut ctx, MeshImportSettings::default()),
            )
            .unwrap(),
    );

    // Create a new material instance
    let material = standard_materials.insert(Standard {
        albedo_map: albedo_map,
        normal_map: normal_map,
        mask_map: mask_map,
        bumpiness: 0.8,
        roughness: 0.8,
        ambient_occlusion: 1.0,
        metallic: 1.0,
        scale: vek::Vec2::broadcast(3.0),
        tint: vek::Rgb::white(),
    });

    // Create a new material surface for rendering
    let pipeid = ctx.material_id::<Standard>().unwrap();    
    let surface = Surface::new(cube.clone(), material.clone(), pipeid);
    ecs.insert((surface, Renderer::default()));

    let surface = Surface::new(cube.clone(), material.clone(), pipeid);
    ecs.insert((surface, Renderer::default(), Location::at_y(-0.5), Scale::scale_xyz(40.0, 1.0, 40.0)));

    let surface = Surface::new(cube.clone(), material.clone(), pipeid);
    ecs.insert((surface, Renderer::default(), Location::at_y(2.5), Scale::scale_xyz(5.0, 5.0, 0.5), Rotation::rotation_z(70.0f32.to_radians())));

    //ecs.insert((Location::at_y(5.0), PointLight::default()));

    // Load in the texture
    let texture = albedo_maps.insert(
        assets
            .load_with::<AlbedoMap>(
                "engine/textures/sky_gradient.png",
                (
                    &mut ctx,
                    TextureImportSettings {
                        sampling: Sampling {
                            filter: Filter::Linear,
                            wrap: Wrap::ClampToEdge,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ),
            )
            .unwrap(),
    );

    // Create the default sky material
    let material = Sky {
        gradient: texture,
        sun_intensity: 15.0,
        sun_size: 1.05,
    };

    // Create the default Sky material pipeline and default Sky sphere surface
    let material = sky_materials.insert(material);
    let pipeid = ctx.material_id::<Sky>().unwrap();
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
    ecs.insert((renderer, surface, Scale::from(vek::Vec3::one() * 400.0)));
}

// We will use this update event to move the camera around
fn update(world: &mut World) {
    let stats = world.get::<RenderedFrameStats>().unwrap();
    let time = world.get::<Time>().unwrap();
    let mut ui = world.get_mut::<UserInterface>().unwrap();
    let ctx = ui.as_mut().as_mut();
    egui::Window::new("Stats").show(ctx, |ui| {
        ui.heading("Timing");
        ui.label(format!("Delta Time MS: {}", (time.delta_f32() * 1000.0).round()));
        ui.label(format!("Startup Timer: {}", time.secs_since_startup_f32().round()));

        ui.heading("Rendering Engine: Surfaces");
        ui.label(format!("Unique Materials: {}", stats.unique_materials));
        ui.label(format!("Material Instances: {}", stats.material_instances));
        ui.label(format!("Rendered Surfaces: {}", stats.rendered_surfaces));
        ui.label(format!("Triangles: {}", stats.tris));
        ui.label(format!("Vertices: {}", stats.verts));

        ui.heading("Rendering Engine: Shadows");
        ui.label(format!("Unique Shadow Caster Materials: {}", stats.unique_materials_shadow_casters));
        ui.label(format!("Shadow Caster Surfaces: {}", stats.shadow_casters_surfaces));
        ui.label(format!("Shadow Caster Triangles: {}", stats.shadow_casters_tris));
        ui.label(format!("Shadow Caster Vertices: {}", stats.shadow_casters_verts));
    });

    let shading = world.get::<ClusteredShading>().unwrap();
    let window = world.get_mut::<Window>().unwrap();

    // Get the input resources
    let input = world.get::<Input>().unwrap();

    if input.key(Key::H).held() {
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
        let (location, rotation) = entry.as_query::<(&mut Location, &mut Rotation)>().unwrap();

        // Forward and right vectors relative to the camera
        let forward = rotation.forward();
        let right = rotation.right();
        let mut velocity = vek::Vec3::<f32>::default();

        // Update the velocity in the forward and backward directions
        if input.key("forward").held() {
            velocity += forward;
        } else if input.key("backward").held() {
            velocity += -forward;
        }

        // Update the velocity in the left and right directions
        if input.key("left").held() {
            velocity += -right;
        } else if input.key("right").held() {
            velocity += right;
        }

        // Update the location with the new velocity
        **location += velocity * time.delta_f32() * SPEED;

        // Calculate a new rotation and apply it
        let pos_x = input.axis("x rotation");
        let pos_y = input.axis("y rotation");
        **rotation = vek::Quaternion::rotation_y(-pos_x as f32 * SENSIVITY)
            * vek::Quaternion::rotation_x(-pos_y as f32 * SENSIVITY);
    }
}
