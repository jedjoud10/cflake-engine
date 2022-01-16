use core::{WriteContext, Context};

use ecs::{system::System, component::ComponentQuery};


// A simple system that we can use as template
fn run(context: Context, components: ComponentQuery) {
    // We want to read the current time from the world
    let read_context = context.read();
    let time = read_context.time.elapsed;
    dbg!(time);
}


// Create the system    
pub fn system(builder: SystemBuilder) {
    builder
        .set_event()
        .link()
        .build();
    let mut system = System::new();
    system.set_event(&mut write.ecs.0, run);
    write.ecs.
}