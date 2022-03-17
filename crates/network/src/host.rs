use std::{
    io::{self, ErrorKind},
    marker::PhantomData,
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, UdpSocket, TcpListener}, collections::HashMap,
};

use getset::{CopyGetters, Getters, MutGetters};
use serde::de::DeserializeOwned;

use crate::{connection::{ConnectedClientId, ConnectedClient}, cache::NetworkCache, sockets::ListenerSockets};

// A host that has multiple clients connect to it
#[derive(Getters, MutGetters)]
pub struct Host {
    // Address
    #[getset(get = "pub")]
    addr: SocketAddrV6,

    // Network cache
    #[getset(get = "pub", get_mut = "pub")]
    cache: NetworkCache,

    // Reading streams
    #[getset(get = "pub")]
    listener: ListenerSockets
}

impl Host {
    // Open a host on a random port
    pub fn open(max_buffer_size: usize, port: Option<u16>) -> Result<Self, io::Error> {
        // Since we are a host, we use the local address
        let local = Ipv6Addr::LOCALHOST;
        // TODO: Learn about flowinfo and scope_id
        let addr = SocketAddrV6::new(local, port.unwrap_or_default(), 0, 0);

        // Make a UDP and TCP listener sockets
        let udp = UdpSocket::bind(addr)?;
        let tcp = TcpListener

        Ok(Self {
            addr,
            cache: NetworkCache::new(max_buffer_size),
            listener: ListenerSockets {
                udp_listen: todo!(),
                tcp_listen: todo!(),
            }
        })
    }
}
