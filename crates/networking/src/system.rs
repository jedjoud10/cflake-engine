
use world::{post_user, user, System, World};
use crate::{NetworkedSession};

// Handle sending / receiving packets and ECS entities
fn tick(world: &mut World) {
    let Ok(mut _session) = world.get_mut::<NetworkedSession>() else {
        return;
    };
    let session = &mut *_session;
}

// This system will automatically insert the input resource and update it each tick
pub fn system(system: &mut System) {
    system.insert_tick(tick).after(post_user);
}
