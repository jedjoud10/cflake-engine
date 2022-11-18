use winit::{event_loop::EventLoop, event::WindowEvent};
use world::{System, user, World, State, post_user};
use crate::prelude::{WindowSettings, GraphicsSettings};


// Insert the required resources
fn init(
    world: &mut World,
    el: &EventLoop<()>,
    window: WindowSettings,
    graphics: GraphicsSettings,
) {
    // Instantiate a new winit window
    let window = crate::context::Window::new(el, window);

    // Create a new Vulkan context
    let graphics = unsafe { crate::context::Graphics::new(&window, graphics) };

    // Add the resources into the world
    world.insert(window);
    world.insert(graphics);
}

// Handle window quitting
fn event(world: &mut World, event: &mut WindowEvent) {
    if matches!(event, WindowEvent::CloseRequested) {
        let mut state = world.get_mut::<State>().unwrap();
        *state = State::Stopped;
    }
}

// Destroy everything
fn shutdown(world: &mut World) {
    let graphics = world.remove::<crate::context::Graphics>();
}


// Context system will just register the Vulkan context and create a simple window
// This system will also handle window events like exiting
pub fn system(system: &mut System, window: WindowSettings, graphics: GraphicsSettings) {
    system.insert_init(move |world: &mut World, el: &EventLoop<()>| 
        init(world, el, window, graphics)
    ).before(user);

    system.insert_window(event);

    system.insert_shutdown(shutdown).after(post_user);
}