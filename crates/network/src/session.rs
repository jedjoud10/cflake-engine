use crate::{client::Client, host::Host};
use enum_as_inner::EnumAsInner;

// Network session
#[derive(EnumAsInner)]
pub enum NetworkSession {
    Host(Host),
    Client(Client),
    None,
}

impl Default for NetworkSession {
    fn default() -> Self {
        Self::None
    }
}

impl NetworkSession {
    // Update the network manager, should be called at the start of every frame, or even before every system execution
    pub fn update(&mut self) -> laminar::Result<()> {
        match self {
            NetworkSession::Host(host) => host.poll()?,
            NetworkSession::Client(client) => client.poll()?,
            NetworkSession::None => { /* No network session */ }
        }
        Ok(())
    }
}
