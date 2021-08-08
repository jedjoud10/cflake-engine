use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::ecs::{Entity, System, SystemState, SystemType};
use crate::engine::core::world::World;
use glam::Vec4Swizzles;

// Create the skysphere system
pub fn create_system(world: &mut World) {
    let mut system = System::default();
    system.name = String::from("Sky System");
    system.link_component::<components::Sky>(world);
    system.link_component::<transforms::Position>(world);
    system.link_component::<transforms::Scale>(world);
    system.entity_loop_event = |entity, world, _| {
        // Set the position of the sky sphere to always be the camera
        let position = world
            .get_entity(world.default_camera_id)
            .get_component::<transforms::Position>(world)
            .position
            .clone();
        *entity
            .get_component_mut::<transforms::Position>(world)
            .position = *position;
    };

    world.add_system(system);
}
