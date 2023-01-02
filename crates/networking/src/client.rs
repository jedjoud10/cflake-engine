use std::{net::{ToSocketAddrs, TcpStream}, io::Read};
use uuid::Uuid;

// A client resource that can be added to the world (as a NetworkedSession)
// Clients can be created when we connect to a specific IP address (Server) 
pub struct Client {
    // Identifier
    uuid: Uuid,

    // Networking
    stream: TcpStream,
}

impl Drop for Client {
    fn drop(&mut self) {
        // Todo: Handle this
        //self.disconnect();
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
        stream.set_nonblocking(true).unwrap();
        log::debug!("Recveived UUID {}", uuid);
        
        Ok(Self {
            uuid,
            stream,
        })
    }

    // Disconnect the client from the server
    pub fn disconnect(self) {
    }

    // Get the client's identifier
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    // Called each networking tick to update the client
    pub(crate) fn tick(&mut self) {

    }
}

// Data transmission
impl Client {
    // Send a message of a specific type to the server
    pub fn send<T>(&mut self, val: T,) {
        todo!()
    }

    // Send a message of a specific type to a specific client
    pub fn message<T>(&mut self, client: Uuid, val: T) {
        todo!()
    }

    // Send a message of a specific type to all the clients
    pub fn broadcast<T>(&mut self, val: T,) {
        todo!()
    }

    // Receive messages of a specific type from the server
    pub fn receive<T>(&mut self) -> &[T] {
        todo!()
    }
}