use std::{
    hash::Hasher,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
};
use uuid::Uuid;

use crate::{Packet, PacketSendError};

// A client resource that can be added to the world (as a NetworkedSession)
// Clients can be created when we connect to a specific IP address (Server)
pub struct Client {
    // Identifier
    uuid: Option<Uuid>,

    // Networking
    stream: TcpStream,
}

impl Drop for Client {
    fn drop(&mut self) {
        self.disconnect();
    }
}

impl Client {
    // Try to connect a client to a server using it's hosted IP address
    // This will halt until the client sucessfully connected
    pub fn connect(address: impl ToSocketAddrs) -> Result<Self, ()> {
        // Try to connect to the server
        let address = address.to_socket_addrs().unwrap().next().unwrap();
        log::debug!("Connecting to address {}...", address);
        let mut stream = TcpStream::connect(address).unwrap();
        log::debug!("Connected to server {} successfully", address);

        // Wait till we receive the appropriate UUID from the server
        let mut bytes = [0u8; 16];
        stream.read(&mut bytes).unwrap();
        let uuid = Uuid::from_bytes(bytes);
        stream.set_nonblocking(false).unwrap();
        log::debug!("Received UUID {}", uuid);

        Ok(Self {
            uuid: Some(uuid),
            stream,
        })
    }

    // Disconnect the client from the server
    pub fn disconnect(&mut self) {}

    // Get the client's identifier (if it's still connected that is)
    pub fn uuid(&self) -> Option<Uuid> {
        self.uuid
    }
}

// Data transmission
impl Client {
    // Send a message of a specific type to a specific client
    // This will send the message to the server and relay it to the client
    pub fn message<T: Packet>(&mut self, client: Uuid, val: T) {
        todo!()
    }

    // Send a message of a specific type to the server
    pub fn send<T: Packet>(&mut self, val: T) -> Result<(), PacketSendError> {
        todo!()
    }

    // Receive messages of a specific type from the clients (or server)
    pub fn receive<T: Packet>(&mut self) -> Vec<T> {
        todo!()
    }
}
