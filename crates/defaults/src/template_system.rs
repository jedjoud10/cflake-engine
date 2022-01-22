use ecs::component::defaults::*;
use ecs::component::*;
use ecs::system::SystemBuilder;
use main::core::{Context, WriteContext};
use main::ecs;

// A simple system that we can use as template
fn run(mut context: Context, components: ComponentQuery) {
    let share = context.share();
    components.update_all_threaded(move |components| {
        let name = components.component::<Name>().unwrap();
        dbg!(&name.name);
        let time = share.read().time.elapsed;
        dbg!(time);
    });
}

// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().set_run_event(run).link::<Name>().build();
}
