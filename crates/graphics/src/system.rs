use crate::{Graphics, GraphicsInit, Window, WindowSettings};
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required graphics resources
fn init(world: &mut World, el: &EventLoop<()>) {
    // Initialization resource
    let init = world.remove::<GraphicsInit>().unwrap();

    // Initialize the Vulkan context and create a winit Window
    let (graphics, window) =
        unsafe { crate::context::init_context_and_window(init, el) };

    // Add the resources to the world
    world.insert(window);
    world.insert(graphics);
}

// Reset the dirty state of the window at the end of each frame
fn update(world: &mut World) {
    let mut window = world.get_mut::<Window>().unwrap();
    window.dirty = false;
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
            let size = vek::Extent2::new(size.width, size.height);
            let mut window = world.get_mut::<Window>().unwrap();
            window.size = size;
            window.dirty = true;
        }

        // Close requested, set the world state to "Stopped"
        WindowEvent::CloseRequested => {
            let mut state = world.get_mut::<State>().unwrap();
            *state = State::Stopped;
        }

        _ => (),
    }
}

// Context system will just register the wgpu context and create a simple window
// This system will also handle window events like exiting
pub fn system(system: &mut System) {
    system
        .insert_init(init)
        .after(utils::threadpool)
        .before(user);

    system.insert_window(event);
    system.insert_update(update).after(post_user);
}
