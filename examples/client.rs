use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .insert_update(update)
        .set_app_name("Client!")
        .execute();
}

// Start a client and connect to the machine hosted server
fn init(world: &mut World) {
    world.insert(NetworkedSession::connect("127.0.0.1:8080").unwrap());
}

fn update(world: &mut World) {
    let input = world.get::<Input>().unwrap();
    let mut session = world.get_mut::<NetworkedSession>().unwrap();

    if input.get_button(Button::A).pressed() {
        if let NetworkedSession::Client(client) = &mut *session {
            client.send::<u32>(5);
        }
    }
}
