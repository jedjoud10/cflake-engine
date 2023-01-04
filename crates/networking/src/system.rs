use world::{post_user, user, System, World};

use crate::NetworkedSession;

// Add networking related resources and update settings
fn init(_world: &mut World) {}

// Handle sending / receiving packets
fn update(world: &mut World) {
    let Some(mut session) = world.get_mut::<NetworkedSession>() else {
        return;
    };

    match &mut *session {
        NetworkedSession::Server(server) => {
            server.tick();
        }
        NetworkedSession::Client(client) => {
            client.tick();
        }
    }
}

// This system will automatically insert the input resource and update it each frame using the window events
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_update(update).after(post_user);
}
