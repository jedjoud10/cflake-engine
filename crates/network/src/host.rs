use std::{
    io::{self, ErrorKind},
    marker::PhantomData,
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, UdpSocket}, collections::HashMap,
};

use getset::{CopyGetters, Getters};
use serde::de::DeserializeOwned;

use crate::{data::deserialize_payload, protocols::EndPoint, connection::{ConnectedClientId, ConnectedClient}};

// A host that has multiple clients connect to it
// In a singleplayer world, the host also contains an internal client
#[derive(Getters)]
pub struct Host {
    #[getset(get = "pub")]
    pub(crate) endpoint: EndPoint,
    // All the connected clients
    connected: HashMap<ConnectedClientId, ConnectedClient>
}

impl Host {
    // Open a host on a random port
    pub fn open() -> Result<Self, io::Error> {
        // Since we are a host, we use the local address
        let local = Ipv6Addr::LOCALHOST;
        // TODO: Learn about flowinfo and scope_id
        let addr = SocketAddrV6::new(local, 0, 0, 0);
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true).unwrap();
        // Get the IpV6 socket address
        let address = match socket.local_addr()? {
            SocketAddr::V4(_) => todo!(),
            SocketAddr::V6(address) => address,
        };
        println!("Host started on port '{:?}'", address);
        Ok(Self {
            endpoint: EndPoint { addr: address, socket },
            connected: HashMap::default(),
        })
    }
}
