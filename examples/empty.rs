use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default().insert_init(|world: &mut World| {}).execute();
}
