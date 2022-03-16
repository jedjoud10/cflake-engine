use std::{
    io::{self, ErrorKind},
    marker::PhantomData,
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, UdpSocket},
};

use getset::CopyGetters;
use serde::{de::DeserializeOwned};

use crate::common::{deserialize_payload};

// A host that has multiple clients connect to it
// In a singleplayer world, the host also contains an internal client
#[derive(CopyGetters)]
pub struct Host {
    socket: UdpSocket,
    #[getset(get_copy = "pub")]
    address: SocketAddrV6,
}

impl Host {
    // Open a host on a random port
    pub fn open() -> Result<Self, io::Error> {
        // Since we are a host, we use the local address
        let local = Ipv6Addr::LOCALHOST;
        // TODO: Learn about flowinfo and scope_id
        let socket = SocketAddrV6::new(local, 0, 0, 0);
        let socket = UdpSocket::bind(socket)?;
        socket.set_nonblocking(true).unwrap();
        // Get the IpV6 socket address
        let address = match socket.local_addr()? {
            SocketAddr::V4(_) => todo!(),
            SocketAddr::V6(address) => address,
        };
        println!("Host started on port '{:?}'", address);
        Ok(Self { socket, address })
    }
}

// Packet receiver
pub struct PacketReceiver<Payload: 'static> {
    socket: UdpSocket,
    _phantom: PhantomData<*const Payload>,
    buffer_size: usize,
    id: u64,
}

impl<Payload: 'static> PacketReceiver<Payload> {
    // Create a new receiver using a host
    pub fn new(host: &Host, id: u64, buffer_size: usize) -> Result<Self, io::Error> {
        let cloned = host.socket.try_clone()?;
        Ok(Self {
            socket: cloned,
            _phantom: Default::default(),
            buffer_size,
            id,
        })
    }
    // Check if we have received any new packets, and return them
    pub fn receive(&self) -> Result<Vec<Payload>, io::Error>
    where
        Payload: DeserializeOwned,
    {
        // Read until the buffer we gen an error
        let mut payloads = Vec::new();
        loop {
            // Simple buffer
            let mut buf = vec![0; self.buffer_size];
            let (_len, _addr) = match self.socket.recv_from(&mut buf) {
                Ok(tuple) => tuple,
                Err(err) => {
                    if err.kind() == ErrorKind::WouldBlock {
                        // Break normally, since this isn't an error theoretically
                        break;
                    } else {
                        // Actual error that needs to be handled
                        return Err(err);
                    }
                }
            };

            // Deserialize the data
            let (_meta, payload) = deserialize_payload(&buf, self.id)?;
            payloads.push(payload);
        }
        Ok(payloads)
    }
}
