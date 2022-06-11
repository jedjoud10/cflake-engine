use glutin::{event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent, DeviceEvent}};
use crate::prelude::*;
use world::World;

// Run the event loop, and start displaying the game engine window
pub(super) fn run(el: EventLoop<()>, systems: Vec<fn(&mut World)>, mut world: World) {
    el.run(move |event, _, cf| {
        match event {
            Event::WindowEvent { window_id: _, event } => window(&mut world, event, cf),
            Event::DeviceEvent { device_id: _, event } => device(&mut world, event, cf),
            Event::MainEventsCleared => update(&mut world, systems.as_ref(), cf),
            _ => (),
        }
    })
}

// Handle new window events
fn window(world: &mut World, event: WindowEvent, cf: &mut ControlFlow) {
    match event {
        WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
        WindowEvent::Resized(size) => {
            let Graphics(device, _) = world.get_mut::<&mut Graphics>().unwrap();
            device.canvas_mut().resize(vek::Extent2::new(size.width as u16, size.height as u16));
        }
        _ => {}
    }
}

// Handle new device events
fn device(world: &mut World, device: DeviceEvent, cf: &mut ControlFlow) {

}


// Execute one step-frame of the engine
fn update(world: &mut World, systems: &[fn(&mut World)], cf: &mut ControlFlow) {
    // We clear the screen at the start of every frame
    let graphics = world.get_mut::<&mut Graphics>().unwrap();
    graphics.0.canvas_mut().clear(Some(vek::Rgb::black()), None, None);

    // Execute the ECS systems in order
    for system in systems {
        system(world)
    }

    // Swap the front and back buffers (OpenGL) so we can actually render something to the screen
    let Graphics(_, ctx) = world.get_mut::<&mut Graphics>().unwrap();
    ctx.raw().swap_buffers().unwrap();
}