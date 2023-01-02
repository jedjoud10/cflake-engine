use crate::{Graphics, Window, WindowSettings, GraphicsInit};
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required graphics resources
fn init(world: &mut World, el: &EventLoop<()>) {
    // Initialization resource
    let init = world.remove::<GraphicsInit>().unwrap();

    // Initialize the Vulkan context and create a winit Window
    let (graphics, window) = unsafe {
        crate::context::init_context_and_window(init, el)
    };

    // Add the window resource to the world
    world.insert(window);

    // Graphics context is global
    *crate::context::CONTEXT.write() = Some(graphics);
}

fn update(world: &mut World) {
    let mut window = world.get_mut::<Window>().unwrap();
    window.dirty = false;
}

// Destroy the underlying Vulkan context when we stop the app
fn shutdown() {
    let taken = crate::context::CONTEXT.write().take().unwrap();
    unsafe { taken.destroy(); }
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
    system.insert_update(update);
    system.insert_shutdown(shutdown).after(post_user);
}
