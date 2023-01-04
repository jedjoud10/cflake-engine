use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .insert_update(update)
        .set_frame_rate_limit(FrameRateLimit::Limited(10))
        .set_app_name("Client!")
        .execute();
}

// Start a client and connect to the machine hosted server
fn init(world: &mut World) {
    world
        .insert(NetworkedSession::connect("127.0.0.1:8080").unwrap());
}

// Send a packet to the server everytime we press the A key
fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let mut session = world.get_mut::<NetworkedSession>().unwrap();

    if time.frame_count() % 10 == 0 {
        if let NetworkedSession::Client(client) = &mut *session {
            let data = time.frame_count() as u32 / 10;
            log::debug!("Send {data} to server");
            client.send::<u32>(data);
        }
    }
}
