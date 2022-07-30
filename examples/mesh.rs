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
        .insert_update(update)
        .execute();
}

// This is an init event that will be called at the start of the game
fn init(world: &mut World) {
    // Get the graphics resources
    let mut ctx = world.get_mut::<Context>().unwrap();
    let mut settings = world.get_mut::<SceneSettings>().unwrap();
    let mut standard_materials = world.get_mut::<Storage<Standard>>().unwrap();
    let mut albedo_maps = world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let mut normal_maps = world.get_mut::<Storage<NormalMap>>().unwrap();
    let mut shaders = world.get_mut::<Storage<Shader>>().unwrap();
    
    // Get the other resources
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let mut keyboard = world.get_mut::<Keyboard>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();

    // Create a perspective camera and insert it into the world as an entity (and update the scene settings)
    let camera = Camera::new(90.0, 0.003, 10000.0, 16.0 / 9.0);
    let camera = ecs.insert((camera, Location::default(), Rotation::default(), Velocity::default()));
    settings.set_main_camera(camera);

    // We will also register some new keybinds for the camera controller
    keyboard.bind("forward", Key::W);
    keyboard.bind("backward", Key::S);
    keyboard.bind("left", Key::A);
    keyboard.bind("right", Key::D);

    // Create a directional light insert it as a light entity (and update the scene settings)
    let light = DirectionalLight::default();
    let entity = ecs
        .insert((light, Rotation::rotation_x(45f32.to_radians())));
    settings.set_main_directional_light(entity);

    // Create some import settings for our albedo and normal map textures
    let import_settings = TextureImportSettings {
        sampling: Sampling {
            filter: Filter::Linear,
            wrap: Wrap::Repeat,
        },
        mipmaps: MipMaps::AutomaticAniso {
            samples: NonZeroU8::new(4).unwrap(),
        },
        mode: TextureMode::Static,
    };

    // Load a normal map texture
    let texture = assets
        .load_with::<NormalMap>("user/textures/normal.png", (&mut ctx, import_settings))
        .unwrap();
    let texture = normal_maps.insert(texture);

    // Create a new material instance with the normal map texture
    let material = Standard::builder()
        .with_normal(&texture)
        .with_bumpiness(1.4)
        .build();
    let material = standard_materials.insert(material);
    
    // Create a new material surface for rendering
    let pipeid = ctx.get_pipe_id::<SpecializedPipeline<Standard>>().unwrap();
    let surface = Surface::new(settings.cube(), material, pipeid);

    // Insert a new entity that contains the valid surface
    ecs.insert((surface, Renderer::default()));
}

#[derive(Component, Default)]
struct Velocity {
    velocity: vek::Vec3<f32>,
}

// We will use this update event to move the camera around
fn update(world: &mut World) {
    // Get the graphic resources
    let settings = world.get::<SceneSettings>().unwrap();
    let window = world.get_mut::<Window>().unwrap();
    
    // Get the input resources
    let keyboard = world.get::<Keyboard>().unwrap();
    let mouse = world.get::<Mouse>().unwrap();

    // Get the other resources
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let time = world.get::<Time>().unwrap();

    // Lock the cursor to the center of the screen
    window.raw().set_cursor_grab(true).unwrap();
    window.raw().set_cursor_visible(false);
    
    if let Some(mut entry) = settings.main_camera().and_then(|c| ecs.entry_mut(c)) {
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