use std::{marker::PhantomData, net::{Ipv6Addr, SocketAddrV6, UdpSocket, SocketAddr}, io::{self, BufReader, Cursor, BufRead, Read}, cell::RefCell};

use getset::CopyGetters;
use serde::{de::DeserializeOwned, Deserialize};

use crate::common::PacketMetadata;

// A host that has multiple clients connect to it
// In a singleplayer world, the host also contains an internal client
#[derive(CopyGetters)]
pub struct Host {
    socket: UdpSocket,
    #[getset(get_copy = "pub")]
    address: SocketAddrV6,
    data: Vec<Vec<u8>>,
}

impl Host {
    // Open a host on the specified port
    pub fn open(port: &str) -> Result<Self, io::Error> {
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
            address,
            data: Default::default(),
        })
    }
}


// Packet receiver
pub struct PacketReceiver<Payload: 'static> {
    socket: UdpSocket,
    _phantom: PhantomData<*const Payload>,
    id: u64,
}

impl<Payload: 'static> PacketReceiver<Payload> {
    // Create a new receiver using a host
    pub fn new(host: &Host, id: u64) -> Result<Self, io::Error> {
        let cloned = host.socket.try_clone()?;
        Ok(Self {
            socket: cloned,
            _phantom: Default::default(),
            id,
        })
    }
    // Check if we have received any new packets, and return them
    pub fn receive(&self) -> Payload where Payload: DeserializeOwned {
        let mut buf = [0; 256];
        self.socket.recv_from(&mut buf).unwrap();
        dbg!(buf);
        // Buf reader
        let cursor = Cursor::new(buf);
        let mut reader = BufReader::new(cursor);
        // Split at the end of the metadata
        let mut metadata = Vec::default();
        let mut payload = Vec::default();
        reader.read_until(0, &mut metadata).unwrap();
        reader.read_until(0, &mut payload).unwrap();
        metadata.pop();
        payload.pop();
        // Deserialize
        let string = String::from_utf8(metadata.clone()).unwrap();
        let metadata = serde_json::from_slice::<PacketMetadata>(&metadata).unwrap();
        let payload = serde_json::from_slice::<Payload>(&payload).unwrap();
        payload
    }
}