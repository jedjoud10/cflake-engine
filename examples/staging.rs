use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .set_app_name("cflake engine staging example")
        .insert_system(first)
        .insert_system(second)
        .insert_init(default)
        .execute();
}

// System that will inserted placeholder events (System A)
fn first(system: &mut System) {
    system
        .insert_init(|_: &mut World| {
            dbg!("System A post-user");
        })
        .after(post_user);
}

// Another system (System B)
fn second(system: &mut System) {
    system
        .insert_init(|_: &mut World| {
            dbg!("System B pre-user");
        })
        .before(user);
}

// Default init event that will run in the normal stage
fn default(_: &mut World) {
    dbg!("Normal init system without explicit stages");
}
