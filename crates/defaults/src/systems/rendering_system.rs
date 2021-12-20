use core::global::{callbacks::{CallbackType::*}, self};

use ecs::{stored::StoredMut, SystemEventType};
use others::callbacks::{OwnedCallback, Callback, RefCallback, MutCallback};
// An improved multithreaded rendering system

// Add the renderer in the render pipeline renderer
fn add_entity(data: &mut (), entity: &ecs::Entity) {
    // Get the internal renderer
    let renderer = global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let renderer_global_id = renderer.global_id;
    let irenderer = renderer.internal_renderer.clone();
    // Get the transform, and make sure it's matrix is valid
    let transform = global::ecs::component::<crate::components::Transform>(entity).unwrap();
    let transform_global_id = transform.global_id;
    let matrix = transform.matrix;
    // Create the shared data
    let shared_data = rendering::SharedData::new((irenderer, matrix));
    let result = rendering::pipec::task(rendering::RenderTask::RendererAdd(shared_data));
    /*
    result.with_callback(GPUObjectCallback(OwnedCallback::new(move |gpuobject| {
        // This callback is called when we actually add the renderer
        match gpuobject {
            rendering::GPUObject::Renderer(renderer_id) => {
                // After adding the renderer, we must update the entity's renderer component using another callback
                global::ecs::world_mut(WorldMut(MutCallback::new(move |world| {
                    let mut r = global::ecs::componentw_mut::<crate::components::Renderer>(renderer_global_id, world);
                    r.internal_renderer.index = Some(renderer_id);
                    // Also update the transform since we're at it
                    let mut t_ = global::ecs::componentw_mut::<crate::components::Transform>(transform_global_id, world);
                    let t = &mut *t_;
                    t.update_matrix();
                })).create());
            },
            _ => {}
        }
    })).create());
    */
    let gpuobject = result.wait_gpuobject();
    // This callback is called when we actually add the renderer
    match gpuobject {
        rendering::GPUObject::Renderer(renderer_id) => {
            // After adding the renderer, we must update the entity's renderer component using another callback
            global::ecs::world_mut(WorldMut(MutCallback::new(move |world| {
                let mut r = global::ecs::componentw_mut::<crate::components::Renderer>(renderer_global_id, world).unwrap();
                r.internal_renderer.index = Some(renderer_id);
                println!("Updated the entity's internal renderer index!");
                // Also update the transform since we're at it
                let mut t_ = global::ecs::componentw_mut::<crate::components::Transform>(transform_global_id, world).unwrap();
                let t = &mut *t_;
                t.update_matrix();
            })).create());
        },
        _ => {}
    }
}
// Remove the renderer from the pipeline renderer
fn remove_entity(data: &mut (), entity: &ecs::Entity) {
    let renderer = global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let index = renderer.internal_renderer.index.unwrap();
    rendering::pipec::task(rendering::RenderTask::RendererRemove(index));
}
// Send the updated data from the entity to the render pipeline as commands
fn update_entity(data: &mut (), entity: &ecs::Entity) {}
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
        system.event(SystemEventType::EntityAdded(add_entity));
        system.event(SystemEventType::EntityUpdate(update_entity));
        system.event(SystemEventType::EntityRemoved(remove_entity));
        system.event(SystemEventType::SystemPrefire(system_prefire));
        // Return the newly made system
        system
    });
}
