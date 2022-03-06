use world::{ecs::event::EventKey, World};

// The rendering system update loop
fn run(world: &mut World, mut data: EventKey) {
    // Render the world
}

// An event fired whenever we add multiple new renderer entities
fn added_entities(world: &mut World, mut data: EventKey) {}

// An event fired whenever we remove multiple renderer entities
fn removed_entities(world: &mut World, mut data: EventKey) {}

// Create the rendering system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .with_run_event(run)
        .with_added_entities_event(added_entities)
        .with_removed_entities_event(removed_entities)
        .link::<crate::components::Renderer>()
        .link::<crate::components::Transform>()
        .build();
}
