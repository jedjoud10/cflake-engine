use cflake_engine::prelude::*;

// Create a game that will draw a simple mesh onto the screen and a movable camera
fn main() {
    App::default()
        .set_window_title("cflake engine mesh example")
        .set_window_fullscreen(true)
        .insert_system(system)
        .execute();
}

// This is an init event that will be called at the start of the game
fn init(world: &mut World) {
    let (ecs, settings, keyboard) = world
        .get_mut::<(&mut EcsManager, &mut SceneSettings, &mut Keyboard)>()
        .unwrap();

    // Create a perspective camera and insert it into the world as an entity (and update the scene settings)
    let camera = Camera::new(90.0, 0.003, 1000.0, 16.0 / 9.0);
    let camera = ecs.insert((camera, Transform::default())).unwrap();
    settings.set_main_camera(camera);

    // We will also register some new keybinds for the camera controller
    keyboard.bind("forward", Key::W);
    keyboard.bind("backward", Key::S);
    keyboard.bind("left", Key::A);
    keyboard.bind("right", Key::D);

    // Load up a new entity renderer and surface nd insert them as a render entity
    let renderer = Renderer::default();
    let surface = Surface::new(settings.cube(), settings.material());
    ecs.insert((renderer, surface, Transform::default()))
        .unwrap();

    // Create a directional light insert it as a light entity (and update the scene settings)
    let light = Directional::default();
    let entity = ecs.insert((light, Transform::looking_down())).unwrap();
    settings.set_main_directional_light(entity);
}

// We will use this update event to move the camera around
fn update(world: &mut World) {
    let (ecs, scene, keyboard, mouse, Graphics(device, _), time) = world
        .get_mut::<(
            &mut EcsManager,
            &SceneSettings,
            &Keyboard,
            &Mouse,
            &mut Graphics,
            &Time,
        )>()
        .unwrap();

    // Lock the cursor basically
    device.window().set_cursor_grab(true).unwrap();
    device.window().set_cursor_visible(false);

    if let Some(mut entry) = scene.main_camera().map(|c| ecs.try_mut_entry(c)).flatten() {
        let transform = entry.get_mut::<Transform>().unwrap();
        let mut velocity = vek::Vec3::<f32>::zero();
        let forward = transform.forward();
        let right = transform.right();
        if keyboard.held("forward") {
            velocity += forward;
        } else if keyboard.held("backward") {
            velocity += -forward;
        }

        if keyboard.held("left") {
            velocity += -right;
        } else if keyboard.held("right") {
            velocity += right;
        }

        transform.position += velocity * time.delta_f32() * 10.0;

        let pos = mouse.position();
        const SENSIVITY: f32 = 0.0007;
        let rot = vek::Quaternion::rotation_y(-pos.x as f32 * SENSIVITY)
            * vek::Quaternion::rotation_x(-pos.y as f32 * SENSIVITY);
        transform.rotation = rot;
    }

    if keyboard.key(Key::H).pressed() {
        println!(
            "Delta {}, FPS: {}",
            time.delta_f32(),
            1.0 / time.delta_f32()
        );
    }
}

// This is an example system that will register specific events
fn system(events: &mut Events) {
    events.registry::<Init>().insert(init);
    events.registry::<Update>().insert(update);
}
