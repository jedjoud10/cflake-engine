use main::{
    core::{Context, WriteContext},
    ecs::component::ComponentQuery,
};

// The physics system update loop
fn run(context: &mut Context, query: ComponentQuery) {
    let read = context.read();
    // Get the world's delta time
    let delta = read.time.delta as f32;

    // For each physics object, we must update the internal physics values and apply them to our transform
    query.update_all_threaded(move |_, components| {
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
    })
}

// Create the physics system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .with_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Physics>()
        .build()
}
