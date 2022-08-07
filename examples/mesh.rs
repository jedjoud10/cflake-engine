use std::num::NonZeroU8;

use cflake_engine::prelude::{*, vek::Lerp};

const ASSETS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/");
const SENSIVITY: f32 = 0.0007;
const SPEED: f32 = 10.0;
const ROTATION_SMOOTH_SPEED: f32 = 30.0;
const VELOCITY_SMOOTH_SPEED: f32 = 30.0;

// Create a game that will draw a simple mesh onto the screen and a movable camera
fn main() {
    App::default()
        .set_window_title("cflake engine mesh example")
        .set_user_assets_folder_path(ASSETS_PATH)
        .insert_init(init)
        //.insert_update(update)
        .execute();
}

// This is an init event that will be called at the start of the game
fn init(world: &mut World) {
    // Get the graphics resources
    let mut ctx = world.get_mut::<Context>().unwrap();
    let mut shading = world.get_mut::<ClusteredShading>().unwrap();
    let mut standard_materials = world.get_mut::<Storage<Standard>>().unwrap();
    let mut albedo_maps = world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let mut normal_maps = world.get_mut::<Storage<NormalMap>>().unwrap();
    let mut mask_maps = world.get_mut::<Storage<MaskMap>>().unwrap();
    let mut shaders = world.get_mut::<Storage<Shader>>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    
    // Get the other resources
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let mut keyboard = world.get_mut::<Keyboard>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();

    // Create a perspective camera and insert it into the world as an entity (and update the scene settings)
    let camera = Camera::new(90.0, 0.003, 10000.0, 16.0 / 9.0);
    let camera = ecs.insert((camera, Location::at_z(5.0), Rotation::default(), Velocity::default()));

    // We will also register some new keybinds for the camera controller
    keyboard.bind("forward", Key::W);
    keyboard.bind("backward", Key::S);
    keyboard.bind("left", Key::A);
    keyboard.bind("right", Key::D);

    // Create a directional light insert it as a light entity (and update the scene settings)
    let light = DirectionalLight::default();
    let entity = ecs
        .insert((light, Rotation::rotation_x(45f32.to_radians())));

    // Import settings for our albedo map textures
    let import_settings_albedo = TextureImportSettings {
        sampling: Sampling::default(),
        mode: TextureMode::default(),
        mipmaps: MipMaps::default(),
    };

    // Import settings for our normal map texture
    let import_settings_normal = TextureImportSettings {
        sampling: Sampling::default(),
        mode: TextureMode::default(),
        mipmaps: MipMaps::default(),
    };

    // Create the default albedo map texture
    let albedo_map = AlbedoMap::new(
        &mut ctx,
        TextureMode::Static,
        vek::Extent2::one(),
        Sampling::default(),
        MipMaps::Disabled,
        &[vek::Vec4::one()]
    ).unwrap();
    let albedo_map = albedo_maps.insert(albedo_map);

    // Create the default normal map texture
    let normal_map = NormalMap::new(
        &mut ctx,
        TextureMode::Static,
        vek::Extent2::one(),
        Sampling::default(),
        MipMaps::Disabled,
        &[vek::Vec3::new(127, 255, 127)]
    ).unwrap();
    let normal_map = normal_maps.insert(normal_map);

    // Create the default mask map texture
    let mask_map = MaskMap::new(
        &mut ctx,
        TextureMode::Static,
        vek::Extent2::one(),
        Sampling::default(),
        MipMaps::Disabled,
        &[vek::Vec2::zero()]
    ).unwrap();
    let mask_map = mask_maps.insert(mask_map);

    let import = MeshImportSettings {
        mode: BufferMode::Static,
        use_normals: true,
        use_tangents: true,
        use_tex_coords: true,
        invert_triangle_ordering: false,
        invert_normals: false,
        invert_tangents: false,
        invert_vertical_tex_coord: false,
        invert_horizontal_tex_coord: false,
        translation: vek::Vec3::zero(),
        rotation: vek::Quaternion::zero(),
        scale: vek::Vec3::one(),
    };

    // Create the default cube primitive mesh
    let cube = assets.load_with::<Mesh>("engine/meshes/cube.obj", (&mut ctx, import)).unwrap();
    let cube = meshes.insert(cube);

    // Create a new material instance with the normal map texture
    let material = standard_materials.insert(Standard { 
        albedo_map,
        normal_map,
        mask_map,
        bumpiness: 1.4,
        roughness: 0.,
        metallic: 0.,
        tint: vek::Rgb::white()
    });

    // Create a new material surface for rendering
    let pipeid = ctx.get_pipe_id::<SpecializedPipeline<Standard>>().unwrap();
    let surface = Surface::new(cube, material, pipeid);

    // Insert a new entity that contains the valid surface
    ecs.insert((surface, Renderer::default()));
}

#[derive(Component, Default)]
struct Velocity {
    velocity: vek::Vec3<f32>,
}

// We will use this update event to move the camera around
fn update(world: &mut World) {
    let shading = world.get::<ClusteredShading>().unwrap();
    let window = world.get_mut::<Window>().unwrap();
    
    // Get the input resources
    let keyboard = world.get::<Keyboard>().unwrap();
    let mouse = world.get::<Mouse>().unwrap();

    // Get the other resources
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let time = world.get::<Time>().unwrap();

    // Lock the cursor to the center of the screen
    //window.raw().set_cursor_grab(true).unwrap();
    //window.raw().set_cursor_visible(false);
    
    if let Some(mut entry) = shading.main_camera().and_then(|c| ecs.entry_mut(c)) {
        // Get the location and rotation since we will update them
        let (location, rotation, velocity) = entry.as_query::<(&mut Location, &mut Rotation, &mut Velocity)>().unwrap();

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

        velocity.velocity = vek::Vec3::lerp_unclamped_precise(velocity.velocity, temp, time.delta_f32() * VELOCITY_SMOOTH_SPEED);

        // Update the location with the new velocity
        **location += velocity.velocity * time.delta_f32() * SPEED;    
        
        // Calculate a new rotation and apply it
        let pos = mouse.position();
        let rot = vek::Quaternion::rotation_y(-pos.x as f32 * SENSIVITY)
            * vek::Quaternion::rotation_x(-pos.y as f32 * SENSIVITY);
        **rotation = vek::Quaternion::lerp_unclamped_precise(**rotation, rot, time.delta_f32() * ROTATION_SMOOTH_SPEED);
    }
}