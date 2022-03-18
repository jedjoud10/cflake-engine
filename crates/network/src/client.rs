use std::{
    net::{SocketAddr, SocketAddrV6},
    thread::JoinHandle,
    time::SystemTime, io::Error,
};

use crate::{serialize_payload, NetworkCache, Payload, PayloadBucketId, registry};
use getset::{Getters, MutGetters};
use laminar::{Packet, Socket, SocketEvent};
use serde::Serialize;
use uuid::Uuid;

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
            sender,
            receiver,
            handle,
            cache: NetworkCache::default(),
            host: addr,
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
    // How we send messages to the server
    pub fn send_unreliable_unordered<P: Payload + 'static>(&self, payload: P) -> Result<(), Error> {
        let bucket_id = registry::get_bucket_id::<P>();
        let packet = Packet::unreliable(self.host, serialize_payload(bucket_id, payload)?);
        self.sender.send(packet).unwrap();
        Ok(())
    }
    pub fn send_reliable_unordered<P: Payload + 'static>(&self, payload: P) -> Result<(), Error> {
        let bucket_id = registry::get_bucket_id::<P>();
        let packet = Packet::reliable_unordered(self.host, serialize_payload(bucket_id, payload)?);
        self.sender.send(packet).unwrap();
        Ok(())
    }
    pub fn send_reliable_ordered<P: Payload + 'static>(&self, payload: P) -> Result<(), Error> {
        let bucket_id = registry::get_bucket_id::<P>();
        let packet = Packet::reliable_ordered(self.host, serialize_payload(bucket_id, payload)?, Some(bucket_id.try_into().unwrap()));
        self.sender.send(packet).unwrap();
        Ok(())
    }
    pub fn send_reliable_sequenced<P: Payload + 'static>(&self, payload: P) -> Result<(), Error> {
        let bucket_id = registry::get_bucket_id::<P>();
        let packet = Packet::reliable_sequenced(self.host, serialize_payload(bucket_id, payload)?, Some(bucket_id.try_into().unwrap()));
        self.sender.send(packet).unwrap();
        Ok(())
    }
    pub fn send_unreliable_sequences<P: Payload + 'static>(&self, payload: P) -> Result<(), Error> {
        let bucket_id = registry::get_bucket_id::<P>();
        let packet = Packet::unreliable_sequenced(self.host, serialize_payload(bucket_id, payload)?, Some(bucket_id.try_into().unwrap()));
        self.sender.send(packet).unwrap();
        Ok(())
    }
}
