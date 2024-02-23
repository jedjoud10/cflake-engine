use crate::scene::{DeferredRenderer, DirectionalLight};
use ecs::Scene;
use graphics::{Graphics, Window};

use world::{system::{post_user, Registries}, world::World, events::Update};

// Update event that will set/update the main directional light
pub fn update(world: &mut World, _: &Update) {
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let _graphics = world.get::<Graphics>().unwrap();
    let mut renderer = world.get_mut::<DeferredRenderer>().unwrap();
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
        let next = ecs.query::<(&DirectionalLight, &coords::Rotation, &ecs::Entity)>();
        if let Some((_, _, entity)) = next.into_iter().next() {
            renderer.main_directional_light = Some(*entity);
        }
    }
}

// The environment system is responsible for creatin the HDRi environment map to use for specular and diffuse IBL
pub fn plugin(registries: &mut Registries) {
    registries.update
        .insert(update)
        .before(super::rendering::update)
        .after(post_user);
}
