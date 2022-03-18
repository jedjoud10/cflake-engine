use std::{collections::HashMap, net::{SocketAddr, Ipv4Addr, Ipv6Addr, SocketAddrV6}, io::Error};

use getset::{Getters, MutGetters};
use laminar::Socket;
use crate::{NetworkCache, ConnectedClientId};

// A host that has multiple clients connect to it
#[derive(Getters, MutGetters)]
pub struct Host {
    // Socket
    #[getset(get = "pub", get_mut = "pub")]
    socket: Socket,

    // Network cache
    #[getset(get = "pub", get_mut = "pub")]
    cache: NetworkCache,
    
    // Connected clients
    connected: HashMap<SocketAddrV6, ConnectedClientId>,
}

impl Host {
    // Start a host on a specific port
    pub fn host(port: Option<u16>) -> laminar::Result<Self> {
        // Create a new laminar socket
        let socket = if let Some(port) = port {
            let ipv6_localhost = SocketAddrV6::new(Ipv6Addr::LOCALHOST, port, 0, 0);
            Socket::bind(ipv6_localhost)
        } else {
            Socket::bind_any()
        }?;
        println!("Server: Bound on port '{}'", socket.local_addr().unwrap().port());
        Ok(Self {
            socket,
            cache: NetworkCache::default(),
            connected: HashMap::default(),
        })
    }
}