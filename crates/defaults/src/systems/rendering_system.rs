use core::global::{callbacks::{CallbackType::EntityRefCallbacks, RefCallback}, self};
// An improved multithreaded rendering system

// Add the renderer in the render pipeline renderer
fn add_entity(data: &mut (), entity: &ecs::Entity) {
    // Get the internal renderer
    let renderer = global::ecs::component::<crate::components::Renderer>(entity);
    let irenderer = renderer.internal_renderer.clone();
    // Get the transform, and make sure it's matrix is valid
    let transform = global::ecs::component::<crate::components::Transform>(entity);
    transform.update_matrix();
    let matrix = transform.matrix;

    let x = match pipec::task_immediate(pipec::RenderTask::RendererAdd(pipec::SharedData::new((irenderer, matrix))), "").unwrap() {
        rendering::RenderTaskReturn::None => todo!(),
        rendering::RenderTaskReturn::GPUObject(x) => match x {
            rendering::GPUObject::Renderer(x) => x,
            _ => panic!()
        },
    };
    entity.get_component_mut::<components::Renderer>(data.component_manager).unwrap().internal_renderer.index = x;
}
// Remove the renderer from the pipeline renderer
fn remove_entity(data: &mut (), entity: &ecs::Entity) {
    let renderer = global::ecs::component::<crate::components::Renderer>(entity);
    let index = renderer.internal_renderer.index;
    let name = rendering::pipec::generate_command_name();
    rendering::pipec::internal_task(rendering::RenderTask::RendererRemove(index), &name);
}
// Send the updated data from the entity to the render pipeline as commands
fn update_entity(data: &mut (), entity: &ecs::Entity) {}
// System prefire so we can send the camera data to the render pipeline
fn system_prefire(data: &mut ()) {
    // Camera data
    let camera = global::ecs::entity(global::main::world_data().main_camera_entity_id).unwrap();
    let camera_data = global::ecs::component::<crate::components::Camera>(&camera);
    // Transform data
    let camera_transform = global::ecs::component::<crate::components::Transform>(&camera);
    let pos = camera_transform.position;
    let rot = camera_transform.rotation;
    let shared_data = rendering::SharedData::new((pos, rot, camera_data.clip_planes, camera_data.projection_matrix));
    rendering::pipec::task(rendering::pipec::RenderTask::CameraDataUpdate(shared_data), "update_camera_data")
}

// Create the default system
pub fn system() {
    ecs::add_system(|| {
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
