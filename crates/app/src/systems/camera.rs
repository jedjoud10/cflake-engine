use crate::prelude::*;

// Camera default init method
fn init(world: &mut World) {
    // Bind inputs to be used by the camera tick event
    let mut input = world.get_mut::<Input>().unwrap();
    input.bind_button("forward", Button::W);
    input.bind_button("backward", Button::S);
    input.bind_button("up", Button::Space);
    input.bind_button("down", Button::LControl);
    input.bind_button("left", Button::A);
    input.bind_button("right", Button::D);
    input.bind_button("lshift", Button::LShift);
    input.bind_button("reset", Button::R);
    input.bind_button("zoom-in", Button::Z);
    input.bind_button("zoom-out", Button::X);
    input.bind_button("toggle-cursor", Button::H);
    input.bind_axis("x rotation", Axis::MousePositionX);
    input.bind_axis("y rotation", Axis::MousePositionY);

    // Make the cursor invisible and locked
    let window = world.get::<Window>().unwrap();
    window
        .raw()
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();
    window.raw().set_cursor_visible(false);
}

// Camera default update method
fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let mut state = world.get_mut::<State>().unwrap();
    let time = &*time;
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    let camera = scene
        .find_mut::<(&mut Camera, &mut Velocity, &mut Position, &mut Rotation)>();
    if let Some((camera, output, position, rotation)) = camera {
        // Forward and right vectors relative to the camera
        let forward = rotation.forward();
        let right = rotation.right();
        let up = rotation.up();
        let mut velocity = vek::Vec3::<f32>::default();
        let mut speed: f32 = 20.0f32;
        let sensivity: f32 = 0.0007;

        // Controls the "strength" of the camera smoothness
        // Higher means more smooth, lower means less smooth
        let smoothness = 0.2;

        // Reset the camera rotation and position
        if input.get_button("reset").pressed() {
            **position = vek::Vec3::zero();
            **output = vek::Vec3::zero();
        }

        // Update velocity scale
        if input.get_button("lshift").held() {
            speed = 120.0f32;
        }

        // Update the velocity in the forward and backward directions
        if input.get_button("forward").held() {
            velocity += forward;
        } else if input.get_button("backward").held() {
            velocity += -forward;
        }

        // Update the velocity in the left and right directions
        if input.get_button("left").held() {
            velocity += -right;
        } else if input.get_button("right").held() {
            velocity += right;
        }

        // Update the velocity in the left and right directions
        if input.get_button("up").held() {
            velocity += up;
        } else if input.get_button("down").held() {
            velocity += -up;
        }

        // Finally multiply velocity by desired speed
        velocity *= speed;

        // Smooth velocity calculation
        let factor = (time.delta().as_secs_f32() * (1.0 / smoothness)).clamped01();
        **output = vek::Vec3::lerp(**output, velocity, (factor * 2.0).clamped01());

        // The scroll wheel OR the X and Z buttons will change the camera FOV
        let mut delta = input.get_axis(Axis::MouseScrollDelta);
        
        // Update based on buttons instead
        if input.get_button("zoom-in").held() {
            delta = 50.0f32;
        } else if input.get_button("zoom-out").held() {
            delta = -50.0f32;
        }

        // Update FOV
        camera.hfov += delta * time.delta().as_secs_f32();

        // Calculate a new rotation and apply it
        let pos_x = input.get_axis("x rotation");
        let pos_y = input.get_axis("y rotation");

        // Smooth rotation
        **rotation = vek::Quaternion::slerp(**rotation, 
            vek::Quaternion::rotation_y(-pos_x * sensivity) *
            vek::Quaternion::rotation_x(-pos_y * sensivity),
            (factor * 5.0).clamped01()
        );
    }
}

// Default camera system innit
pub fn system(system: &mut System) {
    system.insert_init(init);
    system.insert_update(update);
}