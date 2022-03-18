use std::{net::{SocketAddrV6, SocketAddr}, time::SystemTime};

use getset::{Getters, MutGetters};
use laminar::{Socket, Packet};
use crate::{NetworkCache, ConnectionPayload, serialize_payload, PacketMetadata};

// A connected client
pub struct ConnectedClient {
    
}

// Unique identifier for each client that is connected
#[derive(Hash, PartialEq, Eq)]
pub struct ConnectedClientId {
    pub uuid: u64,
}


#[derive(Getters, MutGetters)]
pub struct Client {
    // Sender and receiver
    sender: laminar::,

    // Network cache
    #[getset(get = "pub", get_mut = "pub")]
    cache: NetworkCache,

    // The hosts's address
    host: SocketAddr,
}

impl Client {
    // Create a client and connect it to a host
    pub fn connect(addr: SocketAddr) -> laminar::Result<Self> {
        // Create a new laminar socket for ourselves
        let mut socket = Socket::bind_any()?;
        println!("Client: Bound on port '{}'", socket.local_addr().unwrap().port());

        // Start the poller
        let sender = socket.get_packet_sender();
        let handle = std::thread::spawn(move || socket.start_polling());

        Ok(Self {
            socket,
            cache: NetworkCache::default(),
            host: addr
        })
    }
}