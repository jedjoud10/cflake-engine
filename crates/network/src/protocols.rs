use std::io::Error;

use crate::{Payload, manager::NetworkManager};
mod endpoint;
use self::{udp::{UdpPacketReceiver, UdpPacketSender}, transport::{PacketChannel, PacketTransferParams, PacketDirection}};
pub use endpoint::*;
pub mod tcp;
pub mod transport;
pub mod udp;

// Protocol trait that will be implemented on 
pub trait Protocol<P: Payload> where Self: Sized {
    type Sender;
    type Receiver;
    // Create a channel
    fn new(manager: &mut NetworkManager, params: &PacketTransferParams, direction: PacketDirection) -> Result<PacketChannel<P, Self>, Error>;
    
    // Read from the receiver
    fn recv(receiver: &mut Self::Receiver) -> Result<Vec<P>, Error>;
    // Send using the sender
    fn send(sender: &mut Self::Sender, payload: P) -> Result<usize, Error>;
}
// Udp protocol
pub struct UdpProtocol;

impl<P: Payload + 'static> Protocol<P> for UdpProtocol {
    type Sender = UdpPacketSender<P>;
    type Receiver = UdpPacketReceiver<P>;

    fn new(manager: &mut NetworkManager, params: &PacketTransferParams, direction: PacketDirection) -> Result<PacketChannel<P, Self>, Error> {
        todo!()
    }

    fn recv(receiver: &mut Self::Receiver) -> Result<Vec<P>, Error> {
        todo!()
    }

    fn send(sender: &mut Self::Sender, payload: P) -> Result<usize, Error> {
        todo!()
    }
}


// Tcp protocol
pub struct TcpProtocol;
