use ecs::component::defaults::*;
use ecs::component::*;

use main::core::{Context, WriteContext};
use main::ecs;

// A simple system that we can use as template
fn run(context: &mut Context, query: ComponentQuery) {
    let read = context.read().unwrap();
    let time = read.time.elapsed;
    let obj = read.ecs.get_global::<crate::globals::Terrain>().unwrap();
    query.update_all_threaded(|_, components| {
        let name = components.get_component::<Name>().unwrap();
        dbg!(&name.name);
        dbg!(time);
    });
}

// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).link::<Name>().build();
}
