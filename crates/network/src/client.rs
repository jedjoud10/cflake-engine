use std::{net::{SocketAddrV6, SocketAddr}, time::SystemTime, thread::JoinHandle};

use getset::{Getters, MutGetters};
use laminar::{Socket, Packet, SocketEvent};
use uuid::Uuid;
use crate::{NetworkCache, serialize_payload, PacketBucketId};

// Unique identifier for each client that is connected
#[derive(Hash, PartialEq, Eq)]
pub struct ConnectedClient {
    pub uuid: Uuid,
}


#[derive(Getters, MutGetters)]
pub struct Client {
    // Sender and receiver
    sender: crossbeam_channel::Sender<Packet>,
    receiver: crossbeam_channel::Receiver<SocketEvent>,
    handle: JoinHandle<()>,

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
        println!("Client: Bound on port '{}' & connected to server socket '{}'", socket.local_addr().unwrap().port(), addr);

        // Start polling in another thread
        let sender = socket.get_packet_sender();
        let receiver = socket.get_event_receiver();
        let handle = std::thread::spawn(move || socket.start_polling());

        // Send a single packet to establish a connection
        sender.send(Packet::reliable_unordered(addr, Vec::new())).unwrap();

        Ok(Self {
            sender, receiver, handle,
            cache: NetworkCache::default(),
            host: addr
        })
    }
    // Handle connections and server->client packets
    pub fn poll(&mut self) -> laminar::Result<()> {
        for event in self.receiver.try_iter() {
            match event {
                SocketEvent::Packet(_) => todo!(),
                SocketEvent::Connect(_) => todo!(),
                SocketEvent::Timeout(_) => todo!(),
                SocketEvent::Disconnect(_) => todo!(),
            }
        } 
        Ok(())
    }
    // Send a payload to the server, using a specific packet metadata
}