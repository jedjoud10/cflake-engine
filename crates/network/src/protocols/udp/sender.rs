use std::{
    io::{self, Error},
    marker::PhantomData,
    net::UdpSocket,
};

use crate::{client::Client, data::serialize_payload, PacketMetadata, Payload};

// Packet sender
pub struct UdpPacketSender<P: Payload + 'static> {
    pub(crate) socket: UdpSocket,
    _phantom: PhantomData<*const P>,
    pub(crate) bucket_id: u64,
}

impl<P: Payload + 'static> UdpPacketSender<P> {
}
