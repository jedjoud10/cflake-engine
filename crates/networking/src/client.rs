use std::net::ToSocketAddrs;

use uuid::Uuid;

// A client resource that can be added to the world (as a NetworkedSession)
// Clients can be created when we connect to a specific IP address (Server) 
pub struct Client {
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
        todo!()
    }

    // Disconnect the client from the server
    pub fn disconnect(self) {
        
    }

    // Called each networking tick to update the client
    pub(crate) fn tick(&mut self) {

    }
}

// Data transmission
impl Client {
    // Send a message of a specific type to a specific client
    pub fn send<T>(&mut self, client: Uuid) {
        todo!()
    }

    // Send a message of a specific type to all the clients
    pub fn broadcast<T>(&mut self) {
        todo!()
    }

    // Receive messages of a specific type from the server
    pub fn receive<T>(&mut self) -> &[T] {
        todo!()
    }
}