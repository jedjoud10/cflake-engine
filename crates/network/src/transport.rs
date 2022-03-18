use std::{io::Error, sync::{atomic::AtomicBool, Arc}, marker::PhantomData};
use enum_as_inner::EnumAsInner;
use getset::{CopyGetters, Getters};

use crate::{Payload, ConnectedClientId};

// Either a packet receiver or packet sender
#[derive(EnumAsInner)]
pub enum PacketChannel<P: Payload + 'static> {
    Sender(PacketSender<P>),
    Receiver(PacketReceiver<P>),
}

// Communication direction
pub enum PacketDirection {
    ClientToServer,
    ServerToClient(ConnectedClientId),
}

// Packet sender
#[derive(CopyGetters, Getters)]
pub struct PacketSender<P: Payload + 'static> {
    _phantom: PhantomData<P>,
    #[getset(get_copy = "pub")]
    bucket_id: u64,
}

impl<P: Payload + 'static> PacketSender<P> {
    // Create a new sender using a bucket id
    pub fn new(bucket_id: u64) -> Self {
        Self {
            _phantom: Default::default(),
            bucket_id,
        }
    }
}

// Packet receiver
#[derive(CopyGetters)]
pub struct PacketReceiver<P: Payload + 'static> {
    _phantom: PhantomData<P>,
    #[getset(get_copy = "pub")]
    bucket_id: u64,
}

impl<P: Payload + 'static> PacketReceiver<P> {
    // Create a new receiver using a bucket id
    pub fn new(bucket_id: u64) -> Self {
        Self {
            _phantom: Default::default(),
            bucket_id,
        }
    }
}
