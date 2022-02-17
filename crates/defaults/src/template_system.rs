use ecs::component::defaults::*;

use main::core::World;
use main::ecs;
use main::ecs::event::EventKey;

// A simple system that we can use as template
fn run(world: &mut World, mut data: EventKey) {
    let query = data.as_query_mut().unwrap();
    let time = world.time.elapsed;
    let _obj = world.globals.get_global::<crate::globals::Terrain>().unwrap();
    for (_, components) in query.lock().iter() {
        let name = components.get_component::<Name>().unwrap();
        dbg!(&name.name);
        dbg!(time);
    }
}

// Create the system
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().with_run_event(run).link::<Name>().build();
}
