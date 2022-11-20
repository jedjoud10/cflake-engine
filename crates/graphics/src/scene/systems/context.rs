use crate::prelude::{GraphicSettings, WindowSettings};
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required resources
fn init(
    world: &mut World,
    el: &EventLoop<()>,
    window_settings: WindowSettings,
    graphic_settings: GraphicSettings,
) {
    // Create the winit window
    let raw = crate::context::new_winit_window(el, &window_settings);

    // Create a new Vulkan context
    let graphics = unsafe {
        crate::context::Graphics::new(
            &window_settings.title,
            &raw,
            &graphic_settings,
        )
    };

    // Instantiate a new window wrapper
    let mut window = unsafe {
        crate::context::Window::new(
            window_settings,
            raw,
            &graphics.instance,
            &graphics.entry,
        )
    };

    // Instantiate a new logical device
    let mut device = unsafe {
        crate::context::Device::new(
            &graphic_settings,
            &graphics.instance,
            &graphics.entry,
            &window.surface_loader,
            &window.surface,
        )
    };

    // Create the swapchain
    unsafe {
        crate::context::Window::create_swapchain(
            &mut window,
            &graphics.instance,
            &graphics.entry,
            &device.physical_device,
            &device.logical_device,
            &device.command_pool,
            device.graphics_queue_index,
            &mut device.command_buffers,
        );
    }

    // Add the resources into the world
    world.insert(window);
    world.insert(graphics);
    world.insert(device);
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
    let window = world.remove::<crate::context::Window>().unwrap();
    let device = world.remove::<crate::context::Device>().unwrap();

    unsafe {
        window.destroy(&device)
    };
    unsafe { device.destroy() };
    unsafe { graphics.destroy() };
}

fn update(world: &mut World) {
    let device = world.get_mut::<crate::context::Device>().unwrap();
    let mut window =
        world.get_mut::<crate::context::Window>().unwrap();

    unsafe { window.draw(&device) }
}

// Context system will just register the Vulkan context and create a simple window
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
