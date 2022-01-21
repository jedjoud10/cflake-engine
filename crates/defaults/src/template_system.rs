use main::ecs;
use main::core::Context;
use ecs::component::*;
use ecs::component::defaults::*;
use ecs::system::SystemBuilder;


// A simple system that we can use as template
fn run(mut context: Context, components: ComponentQuery) {
    let share = context.create_shareable_context();
    components.update_all_threaded(move |components| {
        let name = components.component::<Name>().unwrap();
        dbg!(&name.name);
        let time = share.read().time.elapsed;
        dbg!(time);
    });
}


// Create the system    
pub fn system(builder: SystemBuilder<Context>) {
    builder
        .set_event(run)
        .link::<Name>()
        .build();
}