use std::{marker::PhantomData, net::{SocketAddrV6, Ipv6Addr, UdpSocket}, io};
use serde::{Serialize, Deserialize};

use crate::common::{Packet};

// A client identifier 
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ClientId(pub(crate) u64);

// A client that connects to a host
pub struct Client {
    host: SocketAddrV6,
    socket: UdpSocket,
}

impl Client {
    // Create a new client by connecting to a server
    pub fn connect(addr: Ipv6Addr, port: u16) -> Result<Self, io::Error> {
        // Create the localhost socket address
        let local = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0);
        let host = SocketAddrV6::new(addr, port, 0, 0);
        let socket = UdpSocket::bind(local)?;
        socket.connect(host)?;
        println!("Client '{:?}' connected to host '{:?}'", local, host);
        Ok(Self {
            host: host,
            socket: socket,
        })
    } 
}

// Packet sender
pub struct PacketSender<Payload> {
    id: ClientId,
    socket: UdpSocket,
    _phantom: PhantomData<*const Payload>,
}

impl<Payload> PacketSender<Payload> {
    // Create a new sender using a client
    pub fn new(client: &Client) -> Result<Self, io::Error> {
        let cloned = client.socket.try_clone()?;
        Ok(Self {
            id: ClientId(0),
            socket: cloned,
            _phantom: Default::default()
        })
    }
    // Send a packet to the host
    pub fn send(&mut self, payload: Payload) where Payload: Serialize {
        
    }

}