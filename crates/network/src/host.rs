use std::{marker::PhantomData, net::{Ipv6Addr, SocketAddrV6, UdpSocket, SocketAddr}};

use getset::CopyGetters;
use serde::{de::DeserializeOwned, Deserialize};

use crate::common::{Packet};

// A host that has multiple clients connect to it
// In a singleplayer world, the host also contains an internal client
#[derive(CopyGetters)]
pub struct Host {
    socket: UdpSocket,
    #[getset(get_copy = "pub")]
    address: SocketAddrV6,
}

impl Host {
    // Open a host on the specified port
    pub fn open(port: &str) -> Result<Self, std::io::Error> {
        // Since we are a host, we use the local address
        let local = Ipv6Addr::LOCALHOST;
        // TODO: Learn about flowinfo and scope_id
        let socket = SocketAddrV6::new(local, 0, 0,0);
        let socket = UdpSocket::bind(socket)?;
        // Get the IpV6 socket address
        let address = match socket.local_addr()? {
            SocketAddr::V4(_) => todo!(),
            SocketAddr::V6(address) => address,
        };
        println!("Host started on port '{:?}'", address);
        Ok(Self {
            socket,
            address
        })
    }
}


// Packet receiver
pub struct PacketReceiver<Payload> {
    _phantom: PhantomData<*const Payload>,
}

impl<Payload> PacketReceiver<Payload> {
    // Create a new receiver using a host
    pub fn new(host: &Host) -> Self {
        todo!()
    }
    // Check if we have received any new packets, and return them
    pub fn receive<'a, 'de>(&'a self) -> &'a Packet<Payload> where Payload: Deserialize<'de> {
        todo!()
    }
}