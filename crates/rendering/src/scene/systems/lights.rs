use crate::{
    Camera, ForwardRenderer,
};

use ecs::Scene;
use graphics::{
    Graphics, Window,
};


use world::{System, World};

// Update event that will set/update the main directional light
fn update(world: &mut World) {
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let _graphics = world.get::<Graphics>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let _window = world.get::<Window>().unwrap();

    // Fetch the main directioanl light from the scene renderer
    if let Some(entity) = renderer.main_directional_light {
        // Disable the entity in the resource if it got removed
        let _entry = if let Some(entry) = ecs.entry_mut(entity) {
            entry
        } else {
            renderer.main_directional_light = None;
            return;
        };
    } else {
        // Set the main directioanl light if we find one
        let next = ecs.find::<(
            &Camera,
            &ecs::Position,
            &ecs::Rotation,
            &ecs::Entity,
        )>();
        if let Some((_, _, _, entity)) = next {
            renderer.main_camera = Some(*entity);
        }
    }
}

// The camera system will be responsible for updating the camera UBO and matrices
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(super::rendering::system);
}
