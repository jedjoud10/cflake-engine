use crate::{Graphics, WindowSettings};
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
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let queue = graphics.queue();
    /*
    let swapchain = graphics.swapchain();
    let recorder = graphics.acquire();

    unsafe {
        let (index, image) = swapchain.acquire_next_image();
        //recorder.cmd_clear_image(image);
        swapchain.present(queue, (index, image));
    }

    let submission = graphics.submit(recorder);
    submission.wait();
    */
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

    system.insert_update(update).before(user);
    system.insert_window(event);
}
