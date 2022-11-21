use crate::prelude::{GraphicSettings, WindowSettings};
use pollster::FutureExt;
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
    let window = unsafe {
        crate::context::Window::new(
            window_settings.clone(),
            el,
        )
    };

    // Create a new wgpu context
    let graphics = crate::context::Graphics::new(
        &window.raw,
        &graphic_settings,
        &window_settings,
    );

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
    let graphics =
        world.remove::<crate::context::Graphics>().unwrap();
    
    unsafe { graphics.destroy() };
}

// Executed each frame at the start to clear the window
fn update(world: &mut World) {
    let mut context = world.get_mut::<crate::context::Graphics>().unwrap();
    unsafe { context.draw() }
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

    system.insert_window(event);
    system.insert_update(update);

    system.insert_shutdown(shutdown).after(post_user);
}
