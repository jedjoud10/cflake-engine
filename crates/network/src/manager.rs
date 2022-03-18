use crate::{client::Client, host::Host, Payload};
use enum_as_inner::EnumAsInner;

// Session
#[derive(EnumAsInner)]
pub enum Session {
    Networked(NetworkManager),
    Local,
}

// Network manager
#[derive(EnumAsInner)]
pub enum NetworkManager {
    Host(Host),
    Client(Client),
}

impl NetworkManager {
    // Update the network manager, should be called at the start of every frame, or even before every system execution
    pub fn update(&mut self) -> laminar::Result<()> {
        match self {
            NetworkManager::Host(host) => host.poll()?,
            NetworkManager::Client(client) => client.poll()?,
        }
        Ok(())
    }
}