use ecs::component::defaults::*;

use main::core::{Context, WriteContext};
use main::ecs;
use main::ecs::event::EventKey;

// A simple system that we can use as template
fn run(context: &mut Context, data: EventKey) {
    let (mut query, global_fetcher) = data.decompose().unwrap();
    let read = context.read().unwrap();
    let time = read.time.elapsed;
    let _obj = read.ecs.get_global::<crate::globals::Terrain>(&global_fetcher).unwrap();
    for (_, components) in query.lock().iter() {
        let name = components.get_component::<Name>().unwrap();
        dbg!(&name.name);
        dbg!(time);
    }
}

// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).link::<Name>().build();
}
