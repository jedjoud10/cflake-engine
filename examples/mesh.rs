use cflake_engine::prelude::*;

// Create a game that will draw a simple mesh onto the screen
fn main() {
    App::default().insert_system(system).execute();
}

// This is an init event that will be called at the start of the game
fn init(world: &mut World) {
    // Create a perspective camera
    let camera = Camera::new(90.0, 0.003, 1000.0, 16.0/9.0);

    // Fetch main resources
    let (ecs, settings) = world.get_mut::<(&mut EcsManager, &mut SceneSettings)>().unwrap();

    // And insert it into the world as an entity
    ecs.insert(|entity, linker| {
        linker.insert(camera).unwrap();
        linker.insert(Transform::default()).unwrap()
    });

    // Load up a new entity renderer and surface
    let renderer = Renderer::default();
    let surface = Surface::new(settings.cube(), settings.material());

    // And insert them as a render entity
    ecs.insert(|entity, linker| {
        linker.insert(renderer).unwrap();
        linker.insert(surface).unwrap();
        linker.insert(Transform::default()).unwrap();
    });
}

// This is an update event that will be called each frame
fn update(world: &mut World) {

}

// This is an example system that will register specific events
fn system(events: &mut Events) {
    events.registry::<Init>().insert(init);
    events.registry::<Update>().insert(update);
}
