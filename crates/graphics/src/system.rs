use crate::{Graphics, Window, WindowSettings};
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required graphics resources
fn init(world: &mut World, el: &EventLoop<()>) {
    // Initialization resource
    let init = world.get::<WindowSettings>().unwrap().clone();

    // Initialize the WGPU context and create a winit Window
    let (graphics, window) = unsafe { crate::context::init_phobos_context_and_window(init, el) };

    // Add the resources to the world
    world.insert(window);
    world.insert(graphics);
}

// Update the graphics stats based on the current frame data
fn update(world: &mut World) {
    let mut window = world.get_mut::<Window>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
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
            let graphics = world.get::<Graphics>().unwrap();
            window.size = size;
        }

        // Close requested, set the world state to "Stopped"
        WindowEvent::CloseRequested => {
            let mut state = world.get_mut::<State>().unwrap();
            *state = State::Stopped;
        }

        _ => (),
    }
}

// Common system will be responsible for calling the init event and window event
pub fn common(system: &mut System) {
    system
        .insert_update(update)
        .after(post_user)
        .after(acquire)
        .before(present);
    system.insert_init(init).before(user).after(assets::system);
    system.insert_window(event).before(user);
}

// Acquire system will acquire a valid texture to draw to at the start of every frame
pub fn acquire(system: &mut System) {
    system
        .insert_update(|world: &mut World| {
            // TODO: take the texture from the phobos context
        })
        .before(user);
}

// Present system will present the currently acquired texture to the monitor
pub fn present(system: &mut System) {
    system
        .insert_update(|world: &mut World| {
            // TODO: present the texture to the screen
        })
        .after(post_user)
        .after(acquire);
}
