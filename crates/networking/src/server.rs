use std::{collections::HashMap, net::{IpAddr, SocketAddrV4, Ipv4Addr, TcpListener, TcpStream, SocketAddr}, io::Write};
use uuid::Uuid;

// A server resource that can be added to the world (as a NetworkedSession)
// Servers are created by hosting one using the "host" method
pub struct Server {
    // Networking
    listener: TcpListener,

    // Connected clients
    clients: HashMap<Uuid, SocketAddr>,
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
        self.clients.insert(uuid, address);
        stream.write(uuid.as_bytes()).unwrap();
        log::debug!("Sent UUID {}", uuid);
    }

    // Called each networking tick to update the server
    pub(crate) fn tick(&mut self) {
        // Detect newly connected clients
        if let Ok((stream, address)) = self.listener.accept() {
            self.handle_client_connection(stream, address);
        }
    }
}

// Data transmission
impl Server {
    // Send a message of a specific type to a specific client
    pub fn message<T>(&mut self, client: Uuid, val: T,) {
        todo!()
    }

    // Send a message of a specific type to all the clients
    pub fn broadcast<T>(&mut self, val: T,) {
        todo!()
    }

    // Receive messages of a specific type from the clients
    pub fn receive<T>(&mut self) -> &[(T, Uuid)] {
        todo!()
    }
}