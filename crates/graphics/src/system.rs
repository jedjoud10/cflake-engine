use crate::WindowSettings;
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required resources
fn init(
    world: &mut World,
    el: &EventLoop<()>,
    window_settings: WindowSettings,
    app_name: String,
    engine_name: String,
) {
    // Initialize the Vulkan context and create a winit Window
    let (graphics, window) = crate::context::init_context_and_window(
        app_name,
        engine_name,
        el,
        window_settings.clone()
    );

    // Add the resources into the world
    world.insert(window);
    world.insert(graphics);
}

// Handle window quitting and resizing
fn event(world: &mut World, event: &mut WindowEvent) {
    match event {
        // Window has been resized
        WindowEvent::Resized(size) => {
            // Check if the size is valid
            if size.height == 0 || size.height == 0 {
                return;
            }

            // Handle resizing the window
        }

        // Close requested, set the world state to "Stopped"
        WindowEvent::CloseRequested => {
            let mut state = world.get_mut::<State>().unwrap();
            *state = State::Stopped;
        }

        _ => (),
    }
}

// Clear the window at the start of every frame
fn update(world: &mut World) {}

// Context system will just register the wgpu context and create a simple window
// This system will also handle window events like exiting
pub fn system(
    system: &mut System,
    window_settings: WindowSettings,
    app_name: String,
    engine_name: String,
) {
    system
        .insert_init(move |world: &mut World, el: &EventLoop<()>| {
            init(world, el, window_settings, app_name, engine_name)
        })
        .after(utils::threadpool)
        .before(user);

    system.insert_update(update).before(user);
    system.insert_window(event);
}
