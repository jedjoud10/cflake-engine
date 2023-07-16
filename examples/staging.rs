use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .set_app_name("cflake engine staging example")
        .insert_system(a)
        .insert_system(b)
        .insert_system(c)
        .execute();
}

fn a(system: &mut System) {
    system.insert_init(|_: &mut World| {});
}

fn b(system: &mut System) {
    system
        .insert_init(|_: &mut World| {})
        .after(a)
        .after(post_user);
}

fn c(system: &mut System) {
    system
        .insert_init(|_: &mut World| {})
        .before(b)
        .before(user);
}
