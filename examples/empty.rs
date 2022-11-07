use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default().insert_update(|world: &mut World| {
        dbg!("update");
    }).execute();
}
