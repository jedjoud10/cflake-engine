use std::{collections::HashMap, net::{SocketAddr, Ipv4Addr, Ipv6Addr, SocketAddrV6}, io::Error, thread::JoinHandle};

use getset::{Getters, MutGetters, CopyGetters};
use laminar::{Socket, Packet, SocketEvent};
use uuid::Uuid;
use crate::{NetworkCache, ConnectedClient};

// A host that has multiple clients connect to it
#[derive(Getters, MutGetters, CopyGetters)]
pub struct Host {
    // Sender and receiver
    sender: crossbeam_channel::Sender<Packet>,
    receiver: crossbeam_channel::Receiver<SocketEvent>,
    handle: JoinHandle<()>,
    #[getset(get_copy = "pub")]
    local_addr: SocketAddr,

    // Network cache
    #[getset(get = "pub", get_mut = "pub")]
    cache: NetworkCache,
    

    // UUIDs of clients that will connect soon
    uuids: HashMap<SocketAddr, Uuid>,
    // Connected clients
    connected: HashMap<SocketAddr, ConnectedClient>,
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
        let handle = std::thread::spawn(move || socket.start_polling());

        Ok(Self {
            sender, receiver, handle,
            local_addr,
            cache: NetworkCache::default(),
            uuids: HashMap::default(),
            connected: HashMap::default(),
        })
    }
    // Poll all the event (packets, connections) that we must handle
    pub fn poll(&mut self) -> laminar::Result<()> {
        for event in self.receiver.try_iter() {
            match event {
                SocketEvent::Packet(packet) => {
                    // If the client isn't connected, send the payload back to form a connection
                    let client_addr = packet.addr();
                    if !self.connected.contains_key(&client_addr) {
                        // Client isn't connected yet, send back a packet
                        let uuid = uuid::Uuid::new_v4();
                        let payload = uuid.as_bytes().to_vec();
                        self.uuids.insert(client_addr, uuid);
                        self.sender.send(Packet::reliable_unordered(client_addr, payload)).unwrap();
                        // Can't do anything unless we are connected
                        continue;
                    }
                },
                SocketEvent::Connect(client_addr) => {
                    // A client has succsessfully made a connection, we can register them as our own

                    // Simple check just in case
                    assert!(!self.connected.contains_key(&client_addr), "Client address duplication!");
                    assert!(self.uuids.contains_key(&client_addr), "Client UUID not generated!");
                    println!("Server: Client '{}' succsesfully connected", self.uuids.get(&client_addr).unwrap());
                    
                    let uuid = self.uuids.remove(&client_addr).unwrap();
                    self.connected.insert(client_addr, ConnectedClient { uuid });
                },
                SocketEvent::Timeout(client_addr) => {
                    // A client has timed out
                    println!("Server: Client timed out");
                },
                SocketEvent::Disconnect(client_addr) => {
                    // A client has been disconnected
                    assert!(self.connected.contains_key(&client_addr), "Client was not connected in the first place!");
                    let client = self.connected.remove(&client_addr).unwrap();
                    println!("Server: Client '{}' succsesfully disconnected", client.uuid);
                },
            }
        }
        Ok(())
    }
}