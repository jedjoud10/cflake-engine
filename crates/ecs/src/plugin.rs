use world::{world::World, events::{Init, Update, Tick}, system::{Registries, pre_user, post_user}};
use crate::Scene;


/// Init event that will insert the ECS resource
pub fn init(world: &mut World, _: &Init) {
    world.insert(Scene::default());
}

/// At the end of each tick reset the delta states and clear the removed components
pub fn tick(world: &mut World, _: &Tick) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    for (_, archetype) in scene.archetypes_mut() {
        for (_, column) in archetype.table_mut().iter_mut() {
            column.states_mut().reset();
        }
    }

    for (_, vec) in scene.removed.iter_mut() {
        vec.clear();
    }
}

/// Main plugin that will register the init function and clear states tick function
pub fn plugin(registries: &mut Registries) {
    registries.init.insert(init).before(pre_user);
    registries.tick.insert(tick).after(post_user);
}