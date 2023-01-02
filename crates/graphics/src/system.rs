use crate::{Graphics, Window, WindowSettings};
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required resources
fn init(
    world: &mut World,
    el: &EventLoop<()>,
    window_settings: WindowSettings,
    app_name: String,
    app_version: u32,
    engine_name: String,
    engine_version: u32,
) {
    // Initialize the Vulkan context and create a winit Window
    let (graphics, window) = unsafe {
        crate::context::init_context_and_window(
            app_name,
            app_version,
            engine_name,
            engine_version,
            el,
            window_settings.clone(),
        )
    };

    // Add the window resource to the world
    world.insert(window);

    // Graphics context is global
    *crate::context::CONTEXT.write() = Some(graphics);
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

            /*
            unsafe {
                let graphics = world.get_mut::<Graphics>().unwrap();
                graphics.swapchain().recreate(graphics.device(), dimensions);
            }
            */
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
pub fn system(
    system: &mut System,
    window_settings: WindowSettings,
    app_name: String,
    app_version: u32,
    engine_name: String,
    engine_version: u32,
) {
    system
        .insert_init(move |world: &mut World, el: &EventLoop<()>| {
            init(
                world,
                el,
                window_settings,
                app_name,
                app_version,
                engine_name,
                engine_version,
            )
        })
        .after(utils::threadpool)
        .before(user);

    system.insert_window(event);
    system.insert_shutdown(shutdown).after(post_user);
}
