use std::net::{SocketAddrV4, SocketAddr};

use world::{network::{NetworkSession, PayloadCache}, globals::Global};

// Server/Client networking
#[derive(Default, Global)]
pub struct NetworkManager {
    // Either a host or a client
    pub session: Option<NetworkSession>,
    // The IP socket address that we will connect to
    pub host_addr_string: String,
}