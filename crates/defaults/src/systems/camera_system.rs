use world::ecs::event::EventKey;
use world::input::Keys;
use world::World;

// The camera system update loop
fn run(world: &mut World, mut data: EventKey) {
    let query = data.as_query_mut().unwrap();
    // Rotate the camera around
    let mouse_pos = *world.input.get_mouse_position();
    const SENSIVITY: f32 = 0.0007;
    // Create the camera rotation quaternion
    let new_rotation = veclib::Quaternion::<f32>::from_euler_angles(
        veclib::EulerAnglesOrder::YXZ,
        veclib::Vector3::new(-mouse_pos.y as f32 * SENSIVITY, -mouse_pos.x as f32 * SENSIVITY, 0.0),
    );
    // Calculate the vectors
    let forward = new_rotation.mul_point(-veclib::Vector3::<f32>::Z);
    let up = new_rotation.mul_point(veclib::Vector3::<f32>::Y);
    let right = new_rotation.mul_point(veclib::Vector3::<f32>::X);
    let mut velocity: veclib::Vector3<f32> = veclib::Vector3::ZERO;

    // Custom speed
    let original_speed = 0.1 + (*world.input.get_mouse_scroll() as f32 * 0.1).clamp(0.0, 100.0).powf(2.0);
    let speed = original_speed.abs().powf(2.0) * original_speed.signum() * 1.0 * world.time.delta as f32;
    let fov_delta = if world.input.map_held("camera_zoom") {
        1.0
    } else if world.input.map_held("camera_unzoom") {
        -1.0
    } else {
        0.0
    } * world.time.delta as f32
        * 10.0;

    // Actually update the velocity
    // Forward and backward
    if world.input.map_held("camera_forward") {
        velocity += forward * speed;
    } else if world.input.map_held("camera_backwards") {
        velocity += -forward * speed;
    }
    // Left and right
    if world.input.map_held("camera_right") {
        velocity += right * speed;
    } else if world.input.map_held("camera_left") {
        velocity += -right * speed;
    }
    // Up and down
    if world.input.map_held("camera_up") {
        velocity += up * speed;
    } else if world.input.map_held("camera_down") {
        velocity += -up * speed;
    }
    // Update the camera values now
    let mut global = world.globals.get_mut::<crate::globals::GlobalWorldData>().unwrap();
    for (&entity_id, components) in query.write().iter_mut() {
        // If we are not the right camera, skip
        if Some(entity_id) != global.camera_entity_id {
            continue;
        }
        let mut transform = components.get_mut::<crate::components::Transform>().unwrap();
        transform.position += velocity;
        transform.rotation = new_rotation;
        let (position, rotation) = (transform.position, transform.rotation);
        let mut camera = components.get_mut::<crate::components::Camera>().unwrap();
        camera.horizontal_fov += fov_delta;
        // And don't forget to update the camera matrices
        // Load the pipeline since we need to get the window settings
        let pipeline = world.pipeline.read();
        camera.update_aspect_ratio(pipeline.window.dimensions);
        camera.update_view_matrix(position, new_rotation);

        use world::rendering::pipeline;
        let pipeline_camera = world::rendering::pipeline::camera::Camera {
            position,
            rotation,
            forward,
            viewm: camera.view_matrix,
            projm: camera.projection_matrix,
            clip_planes: camera.clip_planes,
        };
        pipeline::pipec::update_callback(&pipeline, |pipeline, _| pipeline.set_internal_camera(pipeline_camera));
        drop(pipeline);

        // Since we are the main camera, we must update our position in the global
        global.camera_pos = position;
        global.camera_forward = forward;
        global.camera_right = right;
        global.camera_up = up;
    }
}

// When we add new cameras
fn added_entities(world: &mut World, mut data: EventKey) {
    let mut global = world.globals.get_mut::<crate::globals::GlobalWorldData>().unwrap();
    // If there isn't a main camera assigned already, we can be the first one
    let query = data.as_query_mut().unwrap();
    if let Some((entity_id, _)) = query.write().iter().nth(0) {
        global.camera_entity_id.get_or_insert(*entity_id);
    }
}

// When we remove old cameras
fn removed_entities(world: &mut World, mut data: EventKey) {
    let mut global = world.globals.get_mut::<crate::globals::GlobalWorldData>().unwrap();
    // If we remove the main camera, we must empty the camera entity ID
    let query = data.as_query_mut().unwrap();
    for (&entity_id, _) in query.write().iter() {
        if Some(entity_id) == global.camera_entity_id {
            // Take
            global.camera_entity_id.take().unwrap();
        }
    }
}

// Create the camera system
pub fn system(world: &mut World) {
    world
        .ecs
        .build_system()
        .with_run_event(run)
        .with_added_entities_event(added_entities)
        .with_removed_entities_event(removed_entities)
        .link::<crate::components::Camera>()
        .link::<crate::components::Transform>()
        .build();
    world.input.bind_key(Keys::W, "camera_forward");
    world.input.bind_key(Keys::S, "camera_backwards");
    world.input.bind_key(Keys::D, "camera_right");
    world.input.bind_key(Keys::A, "camera_left");
    world.input.bind_key(Keys::Space, "camera_up");
    world.input.bind_key(Keys::LShift, "camera_down");
    world.input.bind_key(Keys::Z, "camera_zoom");
    world.input.bind_key(Keys::X, "camera_unzoom");
    world.input.bind_key(Keys::RShift, "cull_update");
}
