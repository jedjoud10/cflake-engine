use std::{
    io::{self, Error, ErrorKind},
    marker::PhantomData,
    net::UdpSocket,
};

use serde::de::DeserializeOwned;

use crate::{data::deserialize_payload, host::Host, protocols::EndPoint, Payload};

// Packet receiver
pub struct UdpPacketReceiver<P: Payload + 'static> {
    socket: UdpSocket,
    _phantom: PhantomData<*const P>,
    buffer_size: usize,
    id: u64,
}

impl<P: Payload + 'static> UdpPacketReceiver<P> {
    // Create a new receiver using an endpoint
    pub fn new(endpoint: &EndPoint, id: u64, buffer_size: usize) -> Result<Self, Error> {
        let cloned = endpoint.socket().try_clone()?;
        Ok(Self {
            socket: cloned,
            _phantom: Default::default(),
            buffer_size,
            id,
        })
    }
    // Check if we have received any new packets, and return them
    pub fn recv(&self) -> Result<Vec<P>, Error>
    where
        P: DeserializeOwned,
    {
        // Read until the buffer we gen an error
        let mut payloads = Vec::new();
        loop {
            // Simple buffer
            let mut buf = vec![0; self.buffer_size];
            let (len, sender) = match self.socket.recv_from(&mut buf) {
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

            // Truncate
            buf.truncate(len);

            // Deserialize the data
            let (_meta, payload) = deserialize_payload(&buf, self.id)?;
            println!("Received '{}' bytes from sender '{:?}'", len, sender);
            payloads.push(payload);
        }
        Ok(payloads)
    }
}
