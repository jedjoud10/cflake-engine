use std::{
    io::{self, Error, ErrorKind},
    marker::PhantomData,
    net::UdpSocket,
};

use serde::de::DeserializeOwned;

use crate::{host::Host, Payload};

// Packet receiver
pub struct UdpPacketReceiver<P: Payload + 'static> {
    _phantom: PhantomData<*const P>,
    pub(crate) bucket_id: u64,
}

impl<P: Payload + 'static> UdpPacketReceiver<P> {
}
