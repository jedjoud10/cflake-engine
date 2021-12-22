use core::global::{self, callbacks::CallbackType::*};

use ecs::SystemEventType;
use others::callbacks::{MutCallback, OwnedCallback, RefCallback};
// An improved multithreaded rendering system

// Add the renderer in the render pipeline renderer
fn entity_added(data: &mut (), entity: &ecs::Entity) {
    // Get the internal renderer
    let renderer = global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let irenderer = renderer.internal_renderer.clone();
    // Get the transform, and make sure it's matrix is valid
    let transform = global::ecs::component::<crate::components::Transform>(entity).unwrap();
    let matrix = transform.matrix;
    // Create the shared data
    let shared_data = rendering::SharedData::new((irenderer, matrix));
    let result = rendering::pipec::task(rendering::RenderTask::RendererAdd(shared_data));
    let entity_id = entity.entity_id;
    result.with_callback(GPUObjectCallback(OwnedCallback::new(move |(_, id)| {
        global::ecs::entity_mut(
            entity_id,
            LocalEntityMut(MutCallback::new(move |entity| {
                let mut r = global::ecs::component_mut::<crate::components::Renderer>(entity).unwrap();
                r.internal_renderer.index = Some(id);
                // Also update the transform since we're at it
                /*
                let t_ = global::ecs::component_mut::<crate::components::Transform>(entity).unwrap();
                let t = &mut *t_;
                t.update_matrix();
                */
            }))
            .create(),
        );
    })).create());    
    /*
    result.with_callback();
    // After adding the renderer, we must update the entity's renderer component using another callback
    global::ecs::entity_mut(
        entity,
        LocalEntityMut(MutCallback::new(move |entity| {
            let mut r = global::ecs::component_mut::<crate::components::Renderer>(entity).unwrap();
            r.internal_renderer.index = Some(gpuobject_id);
            println!("Updated the entity's internal renderer index!");
            // Also update the transform since we're at it
            let mut t_ = global::ecs::component_mut::<crate::components::Transform>(entity).unwrap();
            let t = &mut *t_;
            t.update_matrix();
        }))
        .create(),
    );
    */
}
// Remove the renderer from the pipeline renderer
fn entity_removed(data: &mut (), entity: &ecs::Entity) {
    let renderer = global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let index = renderer.internal_renderer.index.unwrap();
    rendering::pipec::task(rendering::RenderTask::RendererRemove(index));
}
// Send the updated data from the entity to the render pipeline as commands
fn entity_update(data: &mut (), entity: &ecs::Entity) {
    let renderer = core::global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let transform = core::global::ecs::component::<crate::components::Transform>(entity).unwrap();
    match renderer.internal_renderer.index {
        Some(index) => {
            rendering::pipec::task(rendering::RenderTask::RendererUpdateTransform(index, rendering::SharedData::new(transform.matrix)));
        }
        None => {}
    }
}
// System prefire so we can send the camera data to the render pipeline
fn system_prefire(data: &mut ()) {
    // Camera data
    let camera = global::ecs::entity(global::main::world_data().main_camera_entity_id).unwrap();
    let camera_data = global::ecs::component::<crate::components::Camera>(&camera).unwrap();
    // Transform data
    let camera_transform = global::ecs::component::<crate::components::Transform>(&camera).unwrap();
    let pos = camera_transform.position;
    let rot = camera_transform.rotation;
    let shared_data = rendering::SharedData::new((pos, rot, camera_data.clip_planes, camera_data.projection_matrix));
    rendering::pipec::task(rendering::pipec::RenderTask::CameraDataUpdate(shared_data));
}

// Create the default system
pub fn system() {
    core::global::ecs::add_system(|| {
        // Create a system
        let mut system = ecs::System::new(());
        // Link some components to the system
        system.link::<crate::components::Renderer>();
        system.link::<crate::components::Transform>();
        // And link the events
        system.event(SystemEventType::EntityAdded(entity_added));
        system.event(SystemEventType::EntityUpdate(entity_update));
        system.event(SystemEventType::EntityRemoved(entity_removed));
        system.event(SystemEventType::SystemPrefire(system_prefire));
        // Return the newly made system
        system
    });
}
