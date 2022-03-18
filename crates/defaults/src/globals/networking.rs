use world::network::NetworkSession;

// Server/Client networking
pub struct NetworkManager {
    // Either a host or a client
    pub session: NetworkSession,
}