use std::net::{Ipv6Addr, SocketAddrV6, UdpSocket};

use getset::{CopyGetters, Getters};

// An endpoint. Either a host or a client
#[derive(CopyGetters, Getters)]
pub struct EndPoint {
    #[getset(get_copy = "pub")]
    pub(crate) addr: SocketAddrV6,
    #[getset(get = "pub")]
    pub(crate) socket: UdpSocket,
}
