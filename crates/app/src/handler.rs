use crate::prelude::*;
use glutin::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use world::World;

// Run the event loop, and start displaying the game engine window
pub(super) fn run(el: EventLoop<()>, systems: Vec<fn(&mut World)>, mut world: World) {
    el.run(move |event, _, cf| match event {
        Event::WindowEvent {
            window_id: _,
            event,
        } => window(&mut world, event, cf),
        Event::DeviceEvent {
            device_id: _,
            event,
        } => device(&mut world, event, cf),
        Event::MainEventsCleared => update(&mut world, systems.as_ref(), cf),
        _ => (),
    })
}

// Handle new window events
fn window(world: &mut World, event: WindowEvent, cf: &mut ControlFlow) {
    match event {
        WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
        WindowEvent::Resized(size) => {
            let Graphics(device, _) = world.get_mut::<&mut Graphics>().unwrap();

            // If we resize to a null size, just don't do anything
            if size.width > 0 && size.height > 0 {
                device
                .canvas_mut()
                .resize(vek::Extent2::new(size.width as u16, size.height as u16));
            }
        }
        _ => {}
    }
}

// Handle new device events
fn device(_world: &mut World, _device: DeviceEvent, _cf: &mut ControlFlow) {}

// Execute one step-frame of the engine
fn update(world: &mut World, systems: &[fn(&mut World)], _cf: &mut ControlFlow) {
    // Le world is bruh funnier
    world.0.start_frame();

    // We clear the screen at the start of every frame
    let Graphics(device, _) = world.get_mut::<&mut Graphics>().unwrap();
    device.canvas_mut().clear(Some(vek::Rgb::green()), None, None);

    // Execute the ECS systems in order
    for system in systems {
        system(world)
    }

    // Swap the front and back buffers (OpenGL) so we can actually render something to the screen
    let Graphics(_, ctx) = world.get_mut::<&mut Graphics>().unwrap();
    ctx.raw().swap_buffers().unwrap();

    // Indeed funny
    world.0.end_frame();
}
