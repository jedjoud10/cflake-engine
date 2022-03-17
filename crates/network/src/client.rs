use getset::Getters;
use serde::Serialize;
use std::{
    io::{self, Error},
    marker::PhantomData,
    net::{Ipv6Addr, SocketAddrV6, UdpSocket},
};

use crate::{data::serialize_payload, protocols::EndPoint, PacketMetadata, Payload};
// A client that connects to a host
#[derive(Getters)]
pub struct Client {
    #[getset(get = "pub")]
    pub(crate) endpoint: EndPoint,
}

impl Client {
    // Create a new client by connecting to a server
    pub fn connect(addr: SocketAddrV6) -> Result<Self, Error> {
        // Create the localhost socket address
        let local = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0);

        // Client socket
        let socket = UdpSocket::bind(local)?;
        socket.set_nonblocking(true).unwrap();
        socket.connect(addr)?;
        println!("Client '{:?}' connected to host '{:?}'", local, addr);
        Ok(Self {
            endpoint: EndPoint { addr: local, socket: socket },
        })
    }
}
