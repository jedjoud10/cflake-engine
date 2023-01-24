use ecs::{added, Scene, Entity};
use world::{post_user, user, System, World};

use crate::{NetworkedSession, SyncedEntity};

// Add networking related resources and update settings
fn init(_world: &mut World) {}

// Handle sending / receiving packets and ECS entities
fn update(world: &mut World) {
    // Try to get the networked session, and return early if it doesn't exist
    let Ok(mut session) = world.get_mut::<NetworkedSession>() else {
        return;
    };
    
    // Call the "tick" method on both the server and client
    match &mut *session {
        NetworkedSession::Server(server) => {
            server.tick();
        }
        NetworkedSession::Client(client) => {
            client.tick();
        }
    }
    
    // Check for any new SyncedEntities
    let scene = world.get_mut::<Scene>().unwrap();
    let filter = added::<SyncedEntity>();
    let added = scene.query_with::<(&SyncedEntity, &Entity)>(filter);

    // If we're the server, we must send a "SpawnEntity" command to the respective clients
    if let NetworkedSession::Server(server) = &mut *session {
        for (synced, entity) in added {
        }
    }
}

// This system will automatically insert the input resource and update it each frame using the window events
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_update(update).after(post_user);
}
