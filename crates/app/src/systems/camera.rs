use crate::prelude::*;

// Camera controller component that can be modified using GUI
#[derive(Component)]
pub struct CameraController {
    // Camera rotation sensivity
    pub sensivity: f32,

    // Base movement speed
    pub base_speed: f32,

    // Boost speed after pressing the left shift
    pub boost_speed: f32,

    // FOV can be controlled either by scrolling of pressing the Z/X keys
    pub fov_change_scroll_speed: f32,
    pub fov_change_key_speed: f32,

    // Controls the "strength" of the camera smoothness
    // Higher means more smooth, lower means less smooth
    pub smoothness: f32,

    // Is the camera being controlled at the moment?
    pub active: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            sensivity: 1.0,
            base_speed: 20.0,
            boost_speed: 300.0,
            fov_change_scroll_speed: 200.0,
            fov_change_key_speed: 50.0,
            smoothness: 0.2,
            active: true,
        }
    }
}

// Camera default init method
fn init(world: &mut World) {
    // Bind inputs to be used by the camera tick event
    let mut input = world.get_mut::<Input>().unwrap();
    input.bind_button("forward", KeyboardButton::W);
    input.bind_button("backward", KeyboardButton::S);
    input.bind_button("up", KeyboardButton::Space);
    input.bind_button("down", KeyboardButton::LControl);
    input.bind_button("left", KeyboardButton::A);
    input.bind_button("right", KeyboardButton::D);
    input.bind_button("lshift", KeyboardButton::LShift);
    input.bind_button("reset", KeyboardButton::R);
    input.bind_button("zoom-in", KeyboardButton::Z);
    input.bind_button("zoom-out", KeyboardButton::X);
    input.bind_button("toggle-controller", KeyboardButton::H);
    input.bind_axis("x rotation", MouseAxis::PositionX);
    input.bind_axis("y rotation", MouseAxis::PositionY);
}

// Hide the cursor and give control to the camera
fn hide_ui(window: &Window, ui: &mut Interface) {
    ui.consumes_window_events = false;
    window
        .raw()
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();
    window.raw().set_cursor_visible(false);
}

// Show the cursor and give control to the UI
fn show_ui(window: &Window, ui: &mut Interface) {
    ui.consumes_window_events = true;
    window
        .raw()
        .set_cursor_grab(winit::window::CursorGrabMode::None)
        .unwrap();
    window.raw().set_cursor_visible(true);
}

// Camera default update method
fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let time = &*time;
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut gui = world.get_mut::<Interface>().unwrap();
    let window = world.get::<Window>().unwrap();

    let camera = scene.find_mut::<(
        &mut Camera,
        &mut Velocity,
        &mut Position,
        &mut Rotation,
        &mut CameraController,
    )>();

    // Return early if we don't have a camera
    let (camera, output, position, rotation, controller) = if let Some(camera) = camera {
        camera
    } else {
        show_ui(&window, &mut gui);
        return;
    };

    // Decompose the controller settings
    let CameraController {
        sensivity,
        base_speed,
        boost_speed,
        fov_change_scroll_speed,
        fov_change_key_speed,
        smoothness,
        ..
    } = *controller;

    // Forward and right vectors relative to the camera
    let forward = rotation.forward();
    let right = rotation.right();
    let up = rotation.up();
    let mut velocity = vek::Vec3::<f32>::default();

    // Toggle the active state of the controller
    if input.get_button("toggle-controller").pressed() {
        controller.active = !controller.active;
    }

    // If it isn't then exit early
    if !controller.active {
        **output = vek::Vec3::zero();
        show_ui(&window, &mut gui);
        return;
    } else {
        hide_ui(&window, &mut gui);
    }

    // Reset the camera rotation and position
    if input.get_button("reset").pressed() {
        **position = vek::Vec3::zero();
        **output = vek::Vec3::zero();
    }

    // Update velocity scale
    let mut speed = base_speed;
    if input.get_button("lshift").held() {
        speed = boost_speed;
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
    **position += **output * time.delta().as_secs_f32();

    // The scroll wheel OR the X and Z buttons will change the camera FOV
    let mut delta = input.get_axis(MouseAxis::ScrollDelta) * fov_change_scroll_speed;

    // Update based on buttons instead
    if input.get_button("zoom-in").held() {
        delta = fov_change_key_speed;
    } else if input.get_button("zoom-out").held() {
        delta = -fov_change_key_speed;
    }

    // Update FOV
    camera.hfov += delta * time.delta().as_secs_f32();

    // Calculate a new rotation and apply it
    let pos_x = input.get_axis("x rotation");
    let pos_y = input.get_axis("y rotation");

    // Smooth rotation
    **rotation = vek::Quaternion::slerp(
        **rotation,
        vek::Quaternion::rotation_y(-pos_x * sensivity * 0.0007)
            * vek::Quaternion::rotation_x(-pos_y * sensivity * 0.0007),
        (factor * 5.0).clamped01(),
    );

}

// Default camera system innit
pub fn system(system: &mut System) {
    system.insert_init(init);
    system.insert_update(update);
}
