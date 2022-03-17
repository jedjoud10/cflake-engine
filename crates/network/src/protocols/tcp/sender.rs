
use std::{
    io::{self, Error},
    marker::PhantomData,
    net::UdpSocket,
};

use crate::{client::Client, data::serialize_payload, protocols::EndPoint, PacketMetadata, Payload};

// Packet sender
pub struct TcpPacketSender<P: Payload + 'static> {
    socket: UdpSocket,
    _phantom: PhantomData<*const P>,
    id: u64,
}

impl<P: Payload + 'static> TcpPacketSender<P> {
    // Create a new sender using an endpoint
    pub fn new(endpoint: &EndPoint, id: u64) -> Result<Self, Error> {
        todo!()
    }
    // Send a packet to the receiver
    pub fn send(&mut self, ) -> Result<(), Error> {
        todo!()
    }
}
