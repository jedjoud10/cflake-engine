use core::global::{self, callbacks::CallbackType::*};
use std::collections::{HashMap, HashSet};

use ecs::{EntityID, SystemData, SystemEventType};
use others::callbacks::{MutCallback, OwnedCallback, RefCallback};
use rendering::RendererFlags;

// Create a renderer from an entity
fn create_renderer(data: &mut SystemData<()>, entity_id: EntityID, irenderer: &rendering::Renderer, transform: &crate::components::Transform) {
    // The entity is pending
    let data = (irenderer.clone(), transform.calculate_matrix());
    let result = rendering::pipec::task(rendering::RenderTask::RendererAdd(data));
    result.with_callback(
        RenderingGPUObjectCallback(OwnedCallback::new(move |(_, id)| {
            global::ecs::entity_mut(
                entity_id,
                LocalEntityMut(MutCallback::new(move |entity| {
                    if let Ok(x) = global::ecs::component_mut::<crate::components::Renderer>(entity) {
                        // Update the entity's renderer
                        x.internal_renderer.index = Some(id);
                    }
                }))
                .create(),
            );
        }))
        .create(),
    );
}

// Add the renderer in the render pipeline renderer
fn entity_added(data: &mut SystemData<()>, entity: &ecs::Entity) {
    // Get the internal renderer
    let renderer = global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let irenderer = &renderer.internal_renderer;
    // Get the transform, and make sure it's matrix is valid
    let transform = global::ecs::component::<crate::components::Transform>(entity).unwrap();
    // First of all, check if we must auto-add this renderer and check if we have a valid model
    create_renderer(data, entity.id, irenderer, transform);
}
// Remove the renderer from the pipeline renderer
fn entity_removed(data: &mut SystemData<()>, entity: &ecs::Entity) {
    let renderer = global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let index = renderer.internal_renderer.index;
    // Check if the renderer was created on the Render Thread first
    if let Option::Some(index) = index {
        rendering::pipec::task(rendering::RenderTask::RendererRemove(index));
    }
}
// Send the updated data from the entity to the render pipeline as commands
fn entity_update(data: &mut SystemData<()>, entity: &ecs::Entity) {
    let renderer = core::global::ecs::component::<crate::components::Renderer>(entity).unwrap();
    let irenderer = &renderer.internal_renderer;
    let transform = core::global::ecs::component::<crate::components::Transform>(entity).unwrap();
    // Only update the transform if we need to
    if transform.update_frame_id != renderer.update_frame_id {
        match renderer.internal_renderer.index {
            Some(index) => {
                rendering::pipec::task(rendering::RenderTask::RendererUpdateTransform(index, transform.calculate_matrix()));
            }
            None => {}
        }
    }
}

// Create the default system
pub fn system() {
    core::global::ecs::add_system((), || {
        // Create a system
        let mut system = ecs::System::new();
        // Link some components to the system
        system.link::<crate::components::Renderer>();
        system.link::<crate::components::Transform>();
        // And link the events
        system.event(SystemEventType::EntityAdded(entity_added));
        system.event(SystemEventType::EntityUpdate(entity_update));
        system.event(SystemEventType::EntityRemoved(entity_removed));
        // Return the newly made system
        system
    });
}
