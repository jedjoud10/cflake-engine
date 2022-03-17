use std::{io::Error, sync::{atomic::AtomicBool, Arc}};

use enum_as_inner::EnumAsInner;

use crate::{connection::ConnectedClientId, manager::NetworkManager, Payload};

use super::{
    udp::{UdpPacketReceiver, UdpPacketSender},
    EndPoint,
};

// Either a packet receiver or packet sender
#[derive(EnumAsInner)]
pub enum PacketTransporter<P: Payload + 'static> {
    Sender(UdpPacketSender<P>),
    Receiver(UdpPacketReceiver<P>),
}

// Packet transfer parameters
pub struct PacketTransferParams {
    pub id: u64,
    pub max_buffer_size: usize,
}

// Communication direction
pub enum PacketDirection<'a> {
    ClientToServer,
    ServerToClient { client: &'a ConnectedClientId },
}

// Creates a packet transporter bond using a packet direction
pub fn channel<P: Payload + 'static>(manager: &mut NetworkManager, params: &PacketTransferParams, direction: PacketDirection) -> Result<PacketTransporter<P>, Error> {
    match direction {
        PacketDirection::ClientToServer => client_to_server_channel(manager, params),
        PacketDirection::ServerToClient { client } => server_to_client_channel(manager, client, params),
    }
}

// Creates a channel for communication using a network manager (client -> server)
pub fn client_to_server_channel<P: Payload + 'static>(manager: &mut NetworkManager, params: &PacketTransferParams) -> Result<PacketTransporter<P>, Error> {
    // Client -> Server
    match manager {
        NetworkManager::Host(host) => Ok(PacketTransporter::Receiver(UdpPacketReceiver::new(host.endpoint(), params.id, params.max_buffer_size)?)),
        NetworkManager::Client(client) => Ok(PacketTransporter::Sender(UdpPacketSender::new(client.endpoint(), params.id)?)),
    }
}

// Creates a channel for communication using a network manager (server -> client)
pub fn server_to_client_channel<P: Payload + 'static>(
    manager: &mut NetworkManager,
    connected: &ConnectedClientId,
    params: &PacketTransferParams,
) -> Result<PacketTransporter<P>, Error> {
    // server -> client
    match manager {
        NetworkManager::Host(host) => Ok(PacketTransporter::Sender(
            // Todo
            UdpPacketSender::new_send_to(host.endpoint(), params.id, connected)?
            // UdpPacketSender::new(host.endpoint(), params.id)?)
        )),
        NetworkManager::Client(client) => Ok(PacketTransporter::Receiver(UdpPacketReceiver::new(client.endpoint(), params.id, params.max_buffer_size)?)),
    }
}
