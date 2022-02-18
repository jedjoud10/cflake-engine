use main::{core::World, ecs::{event::EventKey, rayon::iter::{IntoParallelRefMutIterator, ParallelIterator}}};

// The physics system update loop
fn run(world: &mut World, mut data: EventKey) {
    let query = data.as_query_mut().unwrap();
    // Get the world's delta time
    let delta = world.time.delta as f32;

    // For each physics object, we must update the internal physics values and apply them to our transform
    query.lock().par_iter_mut().for_each(|(_, components)| {
        // For each physics object, we want to take the transform's position as as a starting point
        let transform = components.get_component::<crate::components::Transform>().unwrap();
        let (position, rotation) = (transform.position, transform.rotation);
        let mut physics = components.get_component_mut::<crate::components::Physics>().unwrap();
        let object = &mut physics.object;
        object.set_position(position);
        object.set_rotation(rotation);
        object.update(delta);
        let (position, rotation) = (*object.get_position(), *object.get_rotation());
        // Apply the physics' object new transform to our current transform
        let mut transform = components.get_component_mut::<crate::components::Transform>().unwrap();
        transform.position = position;
        transform.rotation = rotation;
    });
}

// Create the physics system
pub fn system(world: &mut World) {
    world
        .ecs
        .create_system_builder()
        .with_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Physics>()
        .build()
}
