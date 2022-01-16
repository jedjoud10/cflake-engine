use main::ecs;
use main::core::Context;
use ecs::component::*;
use ecs::component::defaults::*;
use ecs::system::SystemBuilder;


// A simple system that we can use as template
fn run(context: Context, components: ComponentQuery) {
    components.update_all(|components| {
        let name = components.component::<Name>().unwrap();
        dbg!(&name.name);
    }, false);
}


// Create the system    
pub fn system(builder: SystemBuilder<Context>) {
    builder
        .set_event(run)
        .link::<Name>()
        .build();
}