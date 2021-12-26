use core::global::{self, callbacks::CallbackType::*};
use std::collections::{HashMap, HashSet};

use ecs::{SystemEventType, SystemData};
use others::callbacks::{MutCallback, OwnedCallback, RefCallback};
// An improved multithreaded rendering system
#[derive(Default)]
struct RenderingSystem {
    pub invalid_renderers: HashSet<usize> // Renderers that tried to add their renderer in the pipeline, but could not since they have no valid model 
}
ecs::impl_systemdata!(RenderingSystem);

// Create a renderer from an entity
fn create_renderer(data: &mut SystemData<RenderingSystem>, entity_id: usize, irenderer: &rendering::Renderer, transform: &crate::components::Transform) {
    // Create the shared data
    let data = data.clone();
    let shared_data = rendering::SharedData::new((irenderer.clone(), transform.calculate_matrix()));
    let result = rendering::pipec::task(rendering::RenderTask::RendererAdd(shared_data));
    result.with_callback(GPUObjectCallback(OwnedCallback::new(move |(_, id)| {
        global::ecs::entity_mut(
            entity_id,
            LocalEntityMut(MutCallback::new(move |entity| {
                let mut r = global::ecs::component_mut::<crate::components::Renderer>(entity).unwrap();
                // Update the entity's renderer
                r.internal_renderer.index = Some(id);
            }))
            .create(),
        );
    })).create());  
}

// Add the renderer in the render pipeline renderer
fn entity_added(data: &mut SystemData<RenderingSystem>, entity: &ecs::Entity) {
    // Get the internal renderer
    let renderer = global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let irenderer = &renderer.internal_renderer;
    // Get the transform, and make sure it's matrix is valid
    let transform = global::ecs::component::<crate::components::Transform>(entity).unwrap();
    // If the renderer contains an empty model, do not add the renderer
    if irenderer.model.is_none() { 
        data.invalid_renderers.insert(entity.entity_id);
        return;
    }
    // Create the renderer
    create_renderer(data, entity.entity_id, irenderer, transform);     
}
// Remove the renderer from the pipeline renderer
fn entity_removed(data: &mut SystemData<RenderingSystem>, entity: &ecs::Entity) {
    let renderer = global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let index = renderer.internal_renderer.index.unwrap();
    rendering::pipec::task(rendering::RenderTask::RendererRemove(index));
}
// Send the updated data from the entity to the render pipeline as commands
fn entity_update(data: &mut SystemData<RenderingSystem>, entity: &ecs::Entity) {
    let renderer = core::global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let transform = core::global::ecs::component::<crate::components::Transform>(entity).unwrap();
    // Only update the transform if we need to 
    if transform.update_frame_id != renderer.update_frame_id {
        match renderer.internal_renderer.index {
            Some(index) => {
                rendering::pipec::task(rendering::RenderTask::RendererUpdateTransform(index, rendering::SharedData::new(transform.calculate_matrix())));
            }
            None => {}
        }
    }

    // Check if the entity's renderer model has been created, in case it was an invalid renderer
    if renderer.internal_renderer.model.is_some() && data.invalid_renderers.contains(&entity.entity_id) {
        // We can create the renderer for this newly valited entity
        // Get the internal renderer
        let renderer = global::ecs::component::<crate::components::Renderer>(entity).unwrap();
        let irenderer = &renderer.internal_renderer;
        // Get the transform, and make sure it's matrix is valid
        let transform = global::ecs::component::<crate::components::Transform>(entity).unwrap();
        create_renderer(data, entity.entity_id, irenderer, transform); 
        // Remove the invalidation
        data.invalid_renderers.remove(&entity.entity_id);        
    }
}
// System prefire so we can send the camera data to the render pipeline
fn system_prefire(data: &mut SystemData<RenderingSystem>) {
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
    core::global::ecs::add_system(RenderingSystem::default(), || {
        // Create a system
        let mut system = ecs::System::new();
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
