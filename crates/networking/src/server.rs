use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
};
use uuid::Uuid;

use crate::Packet;

// Server side representation of a client
struct ClientRepr {
    uuid: Uuid,
    stream: TcpStream,
    socket_address: SocketAddr,
    data: HashMap<u64, Vec<String>>,
}

// A server resource that can be added to the world (as a NetworkedSession)
// Servers are created by hosting one using the "host" method
pub struct Server {
    // Networking
    listener: TcpListener,

    // Connected clients
    clients: HashMap<Uuid, ClientRepr>,
}

impl Server {
    // Host a new server and let other users join the server
    pub fn host(port: u16) -> Result<Self, ()> {
        // Setup server addresses and ports
        let loopback = Ipv4Addr::new(127, 0, 0, 1);
        let socket = SocketAddrV4::new(loopback, port);
        log::debug!("Hosted server on socket: {socket}");

        // Create a tcp listener
        let listener = TcpListener::bind(socket).unwrap();
        listener.set_nonblocking(true).unwrap();

        Ok(Self {
            listener,
            clients: Default::default(),
        })
    }

    // Handle the connection of a new client
    fn handle_client_connection(&mut self, mut stream: TcpStream, address: SocketAddr) {
        log::debug!("Client {address} has connected to the server");

        // Create a UUID for this client
        let uuid = Uuid::new_v4();
        stream.set_nonblocking(true).unwrap();
        stream.write(uuid.as_bytes()).unwrap();
        log::debug!("Sent UUID {uuid} to client {address}");

        // Add the server side client representation
        self.clients.insert(
            uuid,
            ClientRepr {
                uuid,
                stream,
                socket_address: address,
                data: Default::default(),
            },
        );
    }

    // Handle the disconnection of an old client
    fn handle_client_disconnection(&mut self, uuid: Uuid) {
        let old = self.clients.remove(&uuid).unwrap();
        let address = old.socket_address;
        log::debug!("Client {address} disconnected from the server");
    }

    // Called each networking tick to update the server
    pub(crate) fn tick(&mut self) {
        // Detect newly connected clients
        if let Ok((stream, address)) = self.listener.accept() {
            self.handle_client_connection(stream, address);
        }

        // Clients that we must remove
        let mut disconnected = Vec::<Uuid>::new();

        // Handle client read connections
        for (uuid, client) in self.clients.iter_mut() {
            let mut buf = [0u8; 512];
            if let Ok(len) = client.stream.read(&mut buf) {
                // Check if the client got disconnected
                if len == 0 {
                    disconnected.push(*uuid);
                    continue;
                }

                // Get the TypeID hash in the first 8 bytes of data
                let hash = u64::from_be_bytes(buf[0..8].try_into().unwrap());

                // Read the rest of the data as a string
                let data = (buf[8..][..(len - 8)]).to_vec();
                if let Ok(string) = String::from_utf8(data) {
                    client.data.entry(hash).or_default().push(string);
                }
            }
        }

        // Disconnect the clients
        for client in disconnected {
            self.handle_client_disconnection(client);
        }
    }
}

// Data transmission
impl Server {
    // Send a message of a specific type to a specific client
    pub fn message<T>(&mut self, _client: Uuid, _val: T) {
        todo!()
    }

    // Send a message of a specific type to all the clients
    pub fn broadcast<T>(&mut self, _val: T) {
        todo!()
    }

    // Receive messages of a specific type from the clients
    pub fn receive<T: Packet>(&mut self) -> Vec<(T, Uuid)> {
        let mut output = Vec::new();
        let hash = crate::id::<T>();
        for (uuid, client) in self.clients.iter_mut() {
            let drain = client
                .data
                .entry(hash)
                .or_default()
                .drain(..)
                .filter_map(|x| {
                    let deserialized = serde_json::from_str::<T>(&x);
                    deserialized.map(|x| (x, *uuid)).ok()
                });
            output.extend(drain);
        }
        output
    }
}
