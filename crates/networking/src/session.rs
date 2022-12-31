use crate::{Server, Client};

// This is the main resource that we will use to transfer data between clients and servers
pub enum NetworkedSession {
    // Current machine is acting as a server
    Server(Server),

    // Current machine is acting as a client
    Client(Client),
}