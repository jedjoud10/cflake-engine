use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .set_app_name("cflake engine server example")
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Start hosting a new server
fn init(world: &mut World) {
    world.insert(NetworkedSession::host(8080, None).unwrap());
}

// Receive the packets coming from the clients
fn update(world: &mut World) {
    let mut session = world.get_mut::<NetworkedSession>().unwrap();

    if let NetworkedSession::Server(server) = &mut *session {
        let data = server.receive::<u32>();

        for (data, client) in data {
            log::debug!("Client {client} sent {data}");
        }
    }
}
