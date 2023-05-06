use ecs::{added, Entity, Scene};
use world::{post_user, user, System, World};

use crate::{NetworkedSession, SyncedEntity};

// Add networking related resources and update settings
fn init(_world: &mut World) {}

// Handle sending / receiving packets and ECS entities
fn tick(world: &mut World) {
    // Try to get the networked session, and return early if it doesn't exist
    let Ok(mut session) = world.get_mut::<NetworkedSession>() else {
        return;
    };

    
}

// This system will automatically insert the input resource and update it each tick
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_tick(tick).after(post_user);
}
