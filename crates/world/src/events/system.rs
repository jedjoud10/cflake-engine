use crate::Events;

/*
// System stages are simple containers that store the dependency graph for each stage
#[derive(Default)]
pub struct SystemStages {
}

// Systems are collections of multiple events that we insert onto the world
// Systems can be added onto the current app using the insert_system method
// Systems cannot be inserted more than once, since we keep track of their type internally
pub trait System {
    // Stages for dependencies
    fn stages() -> SystemStages { SystemStages::default() }

    // Main control flow events
    fn init(world: &mut World, el: &EventLoop<()>) {}
    fn update(world: &mut World) {}
    fn shutdown(world: &mut World) {}

    // Custom winit events
    fn device(world: &mut World, device: DeviceEvent) {}
    fn window(world: &mut World, window: WindowEvent) {}
}
*/

// Systems are collections of multiple events that we insert onto the world
// Systems can be added onto the current app using the insert_system method
// Systems cannot be inserted more than once, since we keep track of their type internally
pub trait System: 'static {
    // Consume the system type and insert the corresponding events
    fn insert(self, events: &mut Events);
}

// Implementations of system for fnonce closures and function pointers
impl<F: 'static> System for F
where
    F: FnOnce(&mut Events),
{
    fn insert(self, events: &mut Events) {
        self(events)
    }
}
