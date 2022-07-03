use std::{marker::PhantomData, mem::size_of, net::SocketAddr, thread::JoinHandle, time::Duration};

use crate::{send, NetworkCache, PacketType, Payload, PayloadBucketId};
use getset::{Getters, MutGetters};
use laminar::{Config, Packet, Result, Socket, SocketEvent};
use uuid::Uuid;

#[derive(Getters, MutGetters)]
pub struct Client {
    // Sender and receiver
    sender: crossbeam_channel::Sender<Packet>,
    receiver: crossbeam_channel::Receiver<SocketEvent>,
    _handle: JoinHandle<()>,

    // Network cache
    #[getset(get = "pub", get_mut = "pub")]
    cache: NetworkCache,

    // UUID
    #[getset(get = "pub")]
    uuid: Uuid,

    // The host's address
    host: SocketAddr,

    _phantom: PhantomData<*const ()>,
}

impl Client {
    // Create a client and connect it to a host
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        // Create a new laminar socket for ourselves
        let mut socket = Socket::bind_any_with_config(Config {
            heartbeat_interval: Some(Duration::from_secs(3)),
            ..Default::default()
        })?;
        println!(
            "Client: Bound on port '{}' & connected to server socket '{}'",
            socket.local_addr().unwrap().port(),
            addr
        );

        // Start polling in another thread
        let sender = socket.get_packet_sender();
        let receiver = socket.get_event_receiver();
        let _handle = std::thread::spawn(move || socket.start_polling());

        // Send a single packet to establish a connection
        sender
            .send(Packet::reliable_unordered(addr, Vec::new()))
            .unwrap();

        // Wait till we get a connection back
        if let SocketEvent::Connect(_) = receiver.recv().unwrap() {
        } else {
            // Not good, we didn't get a connection as our first packet
            panic!();
        }
        let uuid = match receiver.recv().unwrap() {
            SocketEvent::Packet(packet) => {
                // Deserialize UUID
                let uuid = packet.payload();
                Uuid::from_bytes(uuid.try_into().unwrap())
            }
            _ => {
                panic!("Did not receive the UUID packet!")
            }
        };

        Ok(Self {
            sender,
            host: addr,
            receiver,
            _handle,
            cache: NetworkCache::default(),
            uuid,
            _phantom: Default::default(),
        })
    }
    // Handle connections and server->client packets
    pub fn poll(&mut self) -> Result<()> {
        for event in self.receiver.try_iter() {
            match event {
                SocketEvent::Packet(packet) => {
                    if packet.payload().len() >= size_of::<PayloadBucketId>() {
                        // Add the data to the network cache
                        self.cache.push(packet);
                    }
                }
                SocketEvent::Connect(_) => panic!("Connection event duplication!"),
                _ => {}
            }
        }
        Ok(())
    }
    // Send a packet to the server using a special packet type
    pub fn send<P: Payload + 'static>(&self, payload: P, _type: PacketType) {
        send(self.host, payload, &self.sender, _type).unwrap();
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // Send a packet to the server to tell it that it must disconnect us
        //self.send(ManagementPayload::Disconnect, PacketType::ReliableOrdered)
    }
}
