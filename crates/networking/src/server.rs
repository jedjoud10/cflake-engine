use std::{collections::HashMap, net::IpAddr};
use uuid::Uuid;

// A server resource that can be added to the world (as a NetworkedSession)
// Servers are created by hosting one using the "host" method
pub struct Server {
    // Connected clients
    clients: HashMap<IpAddr, Uuid>,
}

impl Server {
    // Host a new server and let other users join the server
    pub fn host(port: u16) -> Self {
        todo!()
    }

    // Called each networking tick to update the server
    pub(crate) fn tick(&mut self) {

    }
}

// Data transmission
impl Server {
    // Send a message of a specific type to a specific client
    pub fn send<T>(&mut self, client: Uuid) {
        todo!()
    }

    // Send a message of a specific type to all the clients
    pub fn broadcast<T>(&mut self) {
        todo!()
    }

    // Receive messages of a specific type from the clients
    pub fn receive<T>(&mut self) -> &[(T, Uuid)] {
        todo!()
    }
}