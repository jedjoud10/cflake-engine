use crate::{GraphicSettings, WindowSettings};
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required resources
fn init(
    world: &mut World,
    el: &EventLoop<()>,
    window_settings: WindowSettings,
    graphic_settings: GraphicSettings,
) {
    // Instantiate a new window wrapper
    let window =
        crate::context::Window::new(window_settings.clone(), el);

    // Create a new wgpu context
    let graphics = unsafe {
        crate::context::Graphics::new(
            &window.window(),
            &graphic_settings,
            &window_settings,
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

            // Resize the window by re-configuring WGPU
            let graphics =
                world.get::<crate::context::Graphics>().unwrap();
            //graphics.resize()
            /*
            let config = graphics.config();
            let mut lock = config.lock();
            lock.width = size.width;
            lock.height = size.height;
            graphics.surface().configure(graphics.device(), &*lock);
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

// Clear the window at the start of every frame
fn update(world: &mut World) {
    let mut graphics =
        world.get_mut::<crate::context::Graphics>().unwrap();
    let time = world.get::<time::Time>().unwrap();
    unsafe {
        graphics.draw(time.secs_since_startup_f32().sin().abs());
    }
}

// Destroy the Vulkan context
fn destroy(world: &mut World) {
    let graphics = world.remove::<crate::context::Graphics>().unwrap();
    unsafe { graphics.destroy(); }
    world.remove::<crate::context::Window>().unwrap();
}

// Context system will just register the wgpu context and create a simple window
// This system will also handle window events like exiting
pub fn system(
    system: &mut System,
    window_settings: WindowSettings,
    graphic_settings: GraphicSettings,
) {
    system
        .insert_init(move |world: &mut World, el: &EventLoop<()>| {
            init(world, el, window_settings, graphic_settings)
        })
        .before(user);

    system.insert_update(update).before(user);
    system.insert_shutdown(destroy).after(post_user);
    system.insert_window(event);
}
