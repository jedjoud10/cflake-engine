
use std::{
    io::{self, Error, ErrorKind},
    marker::PhantomData,
    net::UdpSocket,
};

use serde::de::DeserializeOwned;

use crate::{data::deserialize_payload, host::Host, protocols::EndPoint, Payload};

// Packet receiver
pub struct TcpPacketReceiver<P: Payload + 'static> {
    socket: UdpSocket,
    _phantom: PhantomData<*const P>,
    buffer_size: usize,
    id: u64,
}

impl<P: Payload + 'static> TcpPacketReceiver<P> {
    // Create a new receiver using an endpoint
    pub fn new(endpoint: &EndPoint, id: u64, buffer_size: usize) -> Result<Self, Error> {
        todo!()
    }
    // Check if we have received any new packets, and return them
    pub fn recv(&self) -> Result<Vec<P>, Error>
    where
        P: DeserializeOwned,
    {
        todo!()
    }
}
