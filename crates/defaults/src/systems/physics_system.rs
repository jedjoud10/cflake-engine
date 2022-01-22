use main::{core::{WriteContext, Context}, ecs::component::ComponentQuery};

// The physics system update loop
fn run(context: Context, query: ComponentQuery) {
    let read = context.read();
    // Get the world's delta time
    let delta = read.time.delta as f32;

    // For each physics object, we must update the internal physics values and apply them to our transform
    query.update_all_threaded(move |components| {
        let mut physics = components.component_mut::<crate::components::Physics>().unwrap();
        let object = &mut physics.object;
        object.update(delta);
        let (position, rotation) = (*object.get_position(), *object.get_rotation());
        // Apply the physics' object new transform to our current transform
        let mut transform = components.component_mut::<crate::components::Transform>().unwrap();
        transform.position = position; transform.rotation = rotation;
    })
}

// Create the physics system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Physics>()
        .build()
}