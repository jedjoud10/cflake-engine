use std::{io::Error, sync::{atomic::AtomicBool, Arc}};

use enum_as_inner::EnumAsInner;

use crate::{connection::ConnectedClientId, manager::NetworkManager, Payload};

use super::{
    udp::{UdpPacketReceiver, UdpPacketSender},
    Protocol,
};

// Either a packet receiver or packet sender
#[derive(EnumAsInner)]
pub enum PacketChannel<P: Payload + 'static, Proto: Protocol<P>> {
    Sender(Proto::Sender),
    Receiver(Proto::Receiver),
}

// Communication direction
pub enum PacketDirection<'a> {
    ClientToServer,
    ServerToClient { client: &'a ConnectedClientId },
}
/*
// Creates a packet transporter bond using a packet direction
pub fn channel<P: Payload + 'static>(manager: &mut NetworkManager, params: &PacketTransferParams, direction: PacketDirection) -> Result<PacketChannel<P>, Error> {
    match direction {
        PacketDirection::ClientToServer => client_to_server_channel(manager, params),
        PacketDirection::ServerToClient { client } => server_to_client_channel(manager, client, params),
    }
}

// Creates a channel for communication using a network manager (client -> server)
pub fn client_to_server_channel<P: Payload + 'static>(manager: &mut NetworkManager, params: &PacketTransferParams) -> Result<PacketChannel<P>, Error> {
    // Client -> Server
    match manager {
        NetworkManager::Host(host) => Ok(PacketChannel::Receiver(UdpPacketReceiver::new(host.endpoint(), params.id, params.max_buffer_size)?)),
        NetworkManager::Client(client) => Ok(PacketChannel::Sender(UdpPacketSender::new(client.endpoint(), params.id)?)),
    }
}

// Creates a channel for communication using a network manager (server -> client)
pub fn server_to_client_channel<P: Payload + 'static>(
    manager: &mut NetworkManager,
    connected: &ConnectedClientId,
    params: &PacketTransferParams,
) -> Result<PacketChannel<P>, Error> {
    // server -> client
    match manager {
        NetworkManager::Host(host) => Ok(PacketChannel::Sender(
            // Todo
            //UdpPacketSender::new(host.endpoint(), params.id, connected)?
            // UdpPacketSender::new(host.endpoint(), params.id)?)
            todo!()
        )),
        NetworkManager::Client(client) => Ok(PacketChannel::Receiver(UdpPacketReceiver::new(client.endpoint(), params.id, params.max_buffer_size)?)),
    }
}
*/