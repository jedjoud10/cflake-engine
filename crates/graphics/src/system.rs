use crate::{Graphics, Window, WindowSettings};
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required graphics resources
fn init(world: &mut World, el: &EventLoop<()>) {
    // Initialization resource
    let init = world.remove::<WindowSettings>().unwrap();

    // Initialize the WGPU context and create a winit Window
    let (graphics, window) =
        unsafe { crate::context::init_context_and_window(init, el) };

    // Add the resources to the world
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
            let size = vek::Extent2::new(size.width, size.height);
            let mut window = world.get_mut::<Window>().unwrap();
            let graphics = world.get::<Graphics>().unwrap();

            // Update the surface configuration and reconfigure the surface
            window.surface_config.width = size.w;
            window.surface_config.height = size.h;
            let config = &window.surface_config;
            window.surface.configure(graphics.device(), config);
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
    system.insert_init(init).before(user);
    system.insert_window(event).before(user);
}

// Acquire system will acquire a valid texture to draw to at the start of every frame
pub fn acquire(system: &mut System) {
    system
        .insert_update(|world: &mut World| {
            // Acquire a new texture to render to
            let mut window = world.get_mut::<Window>().unwrap();
            let texture =
                window.surface.get_current_texture().unwrap();
            let view = texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            // Set the Window's texture view
            // TODO: Cache the texture views instead?
            window.presentable_texture = Some(texture);
            window.presentable_texture_view = Some(view);

            // Clear the window first, and save the command encoder
        })
        .after(post_user);
}

// Present system will present the currently acquired texture to the monitor
pub fn present(system: &mut System) {
    system
        .insert_update(|world: &mut World| {
            let mut window = world.get_mut::<Window>().unwrap();
            let graphics = world.get::<Graphics>().unwrap();
            graphics.submit_unused(false);
            window.presentable_texture.take().unwrap().present();
        })
        .after(post_user)
        .after(acquire);
}
