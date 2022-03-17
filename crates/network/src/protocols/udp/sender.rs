use std::{
    io::{self, Error},
    marker::PhantomData,
    net::UdpSocket,
};

use crate::{client::Client, data::serialize_payload, protocols::EndPoint, PacketMetadata, Payload};

// Packet sender
pub struct UdpPacketSender<P: Payload + 'static> {
    socket: UdpSocket,
    _phantom: PhantomData<*const P>,
    id: u64,
}

impl<P: Payload + 'static> UdpPacketSender<P> {
    // Create a new sender using an endpoint
    pub fn new(endpoint: &EndPoint, id: u64) -> Result<Self, Error> {
        let cloned = endpoint.socket().try_clone()?;
        Ok(Self {
            socket: cloned,
            _phantom: Default::default(),
            id,
        })
    }
    // Create a new sender using an endpoint
    pub fn new(endpoint: &EndPoint, id: u64) -> Result<Self, Error> {
        let cloned = endpoint.socket().try_clone()?;
        Ok(Self {
            socket: cloned,
            _phantom: Default::default(),
            id,
        })
    }
    // Send a packet to the receiver
    pub fn send(&mut self, payload: P) -> Result<(), Error> {
        // Serialize the data
        let bytes = serialize_payload::<P>(PacketMetadata { id: self.id }, payload)?;
        let len = self.socket.send(&bytes)?;
        println!("Sent '{}' bytes!", len);
        Ok(())
    }
}
