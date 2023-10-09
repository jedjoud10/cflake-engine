use app::prelude::*;

fn main() {
    App::new()
        .insert_init(init)
        .execute().unwrap();
}

fn init(world: &mut World, _: &Init) {
}
