use ecs::component::defaults::*;
use ecs::component::*;

use main::core::{Context, WriteContext};
use main::ecs;

// A simple system that we can use as template
fn run(_context: &mut Context, _query: ComponentQuery) {
    /*
    let read = context.read();
    query.update_all_threaded(|_, components| {
        let name = components.component::<Name>().unwrap();
        dbg!(&name.name);
        let time = read.time.elapsed;
        dbg!(time);
    });
    */
}

// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().with_run_event(run).link::<Name>().build();
}
