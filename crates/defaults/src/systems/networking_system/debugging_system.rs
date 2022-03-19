use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use world::{
    ecs::component::ComponentQuerySet,
    gui::egui,
    network::{Client, Host, NetworkSession, PayloadCache},
    World,
};

use crate::globals::NetworkManager;

// Debug
fn run(world: &mut World, mut _data: ComponentQuerySet) {
    // Simple GUI
    let gui = &world.gui.egui;
    let manager = world.globals.get_mut::<NetworkManager>().unwrap();
    let mut cache: PayloadCache<f32> = PayloadCache::default();
    match &mut manager.session {
        Some(session) => {
            egui::Window::new("Networked Session").show(gui, |ui| {
                // Show the host IP and client UUID
                match session {
                    NetworkSession::Host(host) => {
                        ui.label(format!("Host IP: {}", host.local_addr()));
                        // Also display the UUIDs of each connected client
                        for (_, connected) in host.connected() {
                            ui.label(format!("Client UUID: {}", connected));
                        }
                        host.cache_mut().drain_to_payload_cache(&mut cache);
                        for value in cache.iter() {
                            ui.label(format!("Value: {}", value));
                        }
                    }
                    NetworkSession::Client(client) => {
                        ui.label(format!("Client UUID: {}", client.uuid()));
                        client.send_unreliable_unordered(world.time.elapsed().round()).unwrap();
                    }
                }
            });
        }
        None => {
            egui::Window::new("Networking").show(gui, |ui| {
                // Client server IP lol
                ui.text_edit_singleline(&mut manager.host_addr_string);
                // Convert to an IP
                let ip = manager.host_addr_string.parse::<SocketAddr>();

                if ui.button("Host Session").clicked() {
                    // Start hosting
                    let host = Host::host().unwrap();
                    manager.session = Some(NetworkSession::Host(host));
                    return;
                }

                if let Ok(ip) = ip {
                    if ui.button("Join Session").clicked() {
                        // Try to connect
                        let client = Client::connect(ip).unwrap();
                        manager.session = Some(NetworkSession::Client(client));
                        return;
                    }
                }
            });
        }
    }
}

// Create the networking debug system
pub fn system(world: &mut World) {
    world.ecs.systems.builder(&mut world.events.ecs).event(run).build().unwrap();
}
