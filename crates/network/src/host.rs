use std::{marker::PhantomData, mem::size_of, net::SocketAddr, thread::JoinHandle};

use crate::{send, NetworkCache, PacketType, Payload, PayloadBucketId};
use bimap::BiHashMap;
use getset::{CopyGetters, Getters, MutGetters};
use laminar::{Packet, Socket, SocketEvent};
use uuid::Uuid;

// A host that has multiple clients connect to it
#[derive(Getters, MutGetters, CopyGetters)]
pub struct Host {
    // Sender and receiver
    sender: crossbeam_channel::Sender<Packet>,
    receiver: crossbeam_channel::Receiver<SocketEvent>,
    _handle: JoinHandle<()>,
    #[getset(get_copy = "pub")]
    local_addr: SocketAddr,

    // Network cache
    #[getset(get = "pub")]
    cache: NetworkCache,

    // Connected clients
    #[getset(get = "pub")]
    clients: BiHashMap<SocketAddr, Uuid>,

    _phantom: PhantomData<*const ()>,
}

impl Host {
    // Start a host on a specific port
    pub fn host() -> laminar::Result<Self> {
        // Create a new laminar socket
        let mut socket = Socket::bind_any()?;
        let local_addr = socket.local_addr().unwrap();
        println!("Server: Bound on port '{}'", local_addr.port());

        // Start polling in another thread
        let sender = socket.get_packet_sender();
        let receiver = socket.get_event_receiver();
        let _handle = std::thread::spawn(move || socket.start_polling());

        Ok(Self {
            sender,
            receiver,
            _handle,
            local_addr,
            cache: Default::default(),
            clients: Default::default(),
            _phantom: Default::default(),
        })
    }
    // Poll all the event (packets, connections) that we must handle
    pub fn poll(&mut self) -> laminar::Result<()> {
        for event in self.receiver.try_iter() {
            match event {
                SocketEvent::Packet(packet) => {
                    // If the client isn't connected, send the payload back to form a connection
                    let client_addr = packet.addr();
                    if !self.clients.contains_left(&client_addr) {
                        // Client isn't connected yet, send back a packet
                        let uuid = uuid::Uuid::new_v4();
                        let payload = uuid.as_bytes().to_vec();
                        self.clients.insert(client_addr, uuid);
                        self.sender
                            .send(Packet::reliable_ordered(client_addr, payload.clone(), None))
                            .unwrap();
                        self.sender
                            .send(Packet::reliable_ordered(client_addr, Vec::new(), None))
                            .unwrap();
                        // Can't do anything unless we are connected
                        continue;
                    }
                    println!(
                        "Server: Received packet from Client '{}'",
                        self.clients.get_by_left(&client_addr).unwrap()
                    );
                    if packet.payload().len() >= size_of::<PayloadBucketId>() {
                        // Add the data to the network cache
                        let _bucket_id = self.cache.push(packet);
                    }
                }
                SocketEvent::Connect(client_addr) => {
                    // A client has succsessfully made a connection, we can register them as our own

                    // Simple check just in case
                    assert!(
                        self.clients.contains_left(&client_addr),
                        "Client UUID not generated!"
                    );
                    println!(
                        "Server: Client '{}' succsesfully connected",
                        self.clients.get_by_left(&client_addr).unwrap()
                    );
                }
                SocketEvent::Timeout(_client_addr) => {
                    // A client has timed out
                    println!("Server: Client timed out");
                }
                SocketEvent::Disconnect(client_addr) => {
                    // A client has been disconnected
                    assert!(
                        self.clients.contains_left(&client_addr),
                        "Client was not connected in the first place!"
                    );
                    let (_, uuid) = self.clients.remove_by_left(&client_addr).unwrap();
                    println!("Server: Client '{}' succsesfully disconnected", uuid);
                }
            }
        }

        Ok(())
    }
    // Send a packet to the a specific client using it's UUID
    pub fn send<P: Payload + 'static>(
        &self,
        payload: P,
        _type: PacketType,
        uuid: Uuid,
    ) -> Option<()> {
        // Get the client's socket address
        let addr = self.clients.get_by_right(&uuid)?;
        send(*addr, payload, &self.sender, _type).unwrap();
        Some(())
    }
}
