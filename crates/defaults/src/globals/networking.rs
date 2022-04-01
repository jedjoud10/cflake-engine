use world::{globals::Global, network::NetworkSession};

// Server/Client networking
#[derive(Default, Global)]
pub struct NetworkManager {
    // Either a host or a client, or none
    pub session: NetworkSession,
}
