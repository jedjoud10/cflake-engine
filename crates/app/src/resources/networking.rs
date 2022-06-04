use world::{network::NetworkSession, resources::Resource};

// Server/Client networking
#[derive(Default, Resource)]
pub struct NetworkManager {
    // Either a host or a client, or none
    pub session: NetworkSession,
}
