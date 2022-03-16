use serde::Serialize;
use std::{
    io,
    marker::PhantomData,
    net::{Ipv6Addr, SocketAddrV6, UdpSocket},
};

use crate::common::{serialize_payload, PacketMetadata};
// A client that connects to a host
pub struct Client {
    _host: SocketAddrV6,
    socket: UdpSocket,
}

impl Client {
    // Create a new client by connecting to a server
    pub fn connect(addr: Ipv6Addr, port: u16) -> Result<Self, io::Error> {
        // Create the localhost socket address
        let local = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0);
        let host = SocketAddrV6::new(addr, port, 0, 0);
        let socket = UdpSocket::bind(local)?;
        socket.set_nonblocking(true).unwrap();
        socket.connect(host)?;
        println!("Client '{:?}' connected to host '{:?}'", local, host);
        Ok(Self { _host: host, socket: socket })
    }
}

// Packet sender
pub struct PacketSender<Payload: 'static> {
    socket: UdpSocket,
    _phantom: PhantomData<*const Payload>,
    id: u64,
}

impl<Payload: 'static> PacketSender<Payload> {
    // Create a new sender using a client
    pub fn new(client: &Client, id: u64) -> Result<Self, io::Error> {
        let cloned = client.socket.try_clone()?;
        Ok(Self {
            socket: cloned,
            _phantom: Default::default(),
            id,
        })
    }
    // Send a packet to the host
    pub fn send(&mut self, payload: Payload) -> Result<(), io::Error>
    where
        Payload: Serialize,
    {
        // Serialize the data
        let bytes = serialize_payload::<Payload>(PacketMetadata { id: self.id }, payload)?;
        self.socket.send(&bytes)?;
        Ok(())
    }
}
