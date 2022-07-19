use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default().execute();
}

struct Test {
    prop: u32
}

fn test(world: &mut World) {
    let _t = world.get_mut::<Test>().unwrap();
}