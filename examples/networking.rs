use std::cell::RefCell;

use cflake_engine::{defaults::{systems::networking_system, globals::NetworkManager}, ecs::component::ComponentQuerySet, *, network::{NetworkSession, PayloadCache, PacketType}, gui::egui::{self, Slider}, globals::Global};

// An empty game window
fn main() {
    cflake_engine::start("cflake-examples", "networking", init, |world| {
        networking_system::debugging_system::system(world);

        // Messenger
        world.globals.insert(ClientMessenger { value: Default::default() }).unwrap();
        world.ecs.systems.builder(&mut world.events.ecs).event(run).build().unwrap();
    })
}

// Simple global to store a client's message
#[derive(Global)]
struct ClientMessenger {
    value: RefCell<i32>,
}

// Create a system that will send / receive messages
fn run(world: &mut World, _data: ComponentQuerySet) {
    // Fetch from world
    let manager = world.globals.get::<NetworkManager>().unwrap();
    let messenger = world.globals.get::<ClientMessenger>().unwrap();
    let context = &mut world.gui.egui;

    if let Some(session) = &manager.session {
        match session {
            NetworkSession::Host(host) => {
                // Create a payload cache where we can store all the received payloads
                let mut payloads = PayloadCache::<i32>::default();

                // Drain the packets into the payload cache
                host.cache().drain_bucket_to_payload_cache(&mut payloads);

                if let Some(message) = payloads.newest() {
                    println!("{}", message);
                }
            },
            NetworkSession::Client(client) => {
                // Send a message to the host
                egui::Window::new("Send message to host").show(context, |ui| {
                    let mut value = messenger.value.borrow_mut();
                    // Text field for the message 
                    ui.add(Slider::new(&mut *value, -100..=100));
                    
                    if ui.button("Send Message to Host").clicked() {
                        client.send(*value, PacketType::ReliableUnordered);
                    }
                });
            },
        }
    }
}

// Init the empty world
fn init(_world: &mut World) {}
