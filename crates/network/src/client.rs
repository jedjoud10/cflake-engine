use getset::Getters;
use serde::Serialize;
use std::{
    io::{self, Error},
    marker::PhantomData,
    net::{Ipv6Addr, SocketAddrV6, UdpSocket},
};

use crate::{data::serialize_payload, PacketMetadata, Payload, cache::NetworkCache, sockets::StreamSockets};
// A client that connects to a host
#[derive(Getters)]
pub struct Client {
    // Address
    #[getset(get = "pub")]
    addr: SocketAddrV6,

    // Network cache
    #[getset(get = "pub")]
    cache: NetworkCache,

    // Writing streams
    #[getset(get = "pub")]
    stream: StreamSockets,
}

impl Client {
    // Create a new client by connecting to a server
    pub fn connect(addr: SocketAddrV6, max_buffer_size: usize) -> Result<Self, Error> {
        // Create the localhost socket address
        let local = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0);
        Ok(Self {
            addr: local,
            cache: NetworkCache::new(max_buffer_size),
            stream: StreamSockets {
                udp_stream: todo!(),
                tcp_stream: todo!(),
            }
        })
    }
}
