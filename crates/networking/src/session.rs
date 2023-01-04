use std::net::ToSocketAddrs;

use crate::{Client, Server};

// This is the main resource that we will use to transfer data between clients and servers
pub enum NetworkedSession {
    // Current machine is acting as a server
    Server(Server),

    // Current machine is acting as a client
    Client(Client),
}

impl NetworkedSession {
    // Host a new networked session as a server
    pub fn host(port: u16) -> Result<Self, ()> {
        Server::host(port).map(NetworkedSession::Server)
    }

    // Connect to a server as a client
    pub fn connect(address: impl ToSocketAddrs) -> Result<Self, ()> {
        Client::connect(address).map(NetworkedSession::Client)
    }
}
