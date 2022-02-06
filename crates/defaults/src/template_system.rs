use ecs::component::defaults::*;

use main::core::{Context, WriteContext};
use main::ecs;
use main::ecs::event::EventKey;

// A simple system that we can use as template
fn run(context: &mut Context, data: EventKey) {
    let mut query = data.get_query().unwrap();
    let read = context.read().unwrap();
    let time = read.time.elapsed;
    let _obj = read.globals.get_global::<crate::globals::Terrain>().unwrap();
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
