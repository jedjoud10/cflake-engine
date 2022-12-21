use world::{World, user, post_user, System};

// Add networking related resources and update settings
fn init(world: &mut World) {
}

// Handle sending / receiving packets
fn update(world: &mut World) {
}

// This system will automatically insert the input resource and update it each frame using the window events
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_update(update).after(post_user);
}