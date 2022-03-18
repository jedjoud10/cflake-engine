use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr};

use world::{ecs::component::ComponentQuerySet, World, gui::egui, network::{Client, NetworkSession, Host}};

use crate::globals::NetworkManager;

// Update the network manager (if we have one)
fn run(world: &mut World, mut _data: ComponentQuerySet) {
    // Simple GUI
    let gui = &world.gui.egui;
    let manager = world.globals.get_mut::<NetworkManager>().unwrap();
    match &mut manager.session {
        Some(session) => {
        },
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
        },
    }    
}

// Create the networking system
pub fn system(world: &mut World) {
    world.ecs.systems.builder(&mut world.events.ecs).event(run).build().unwrap();
}
