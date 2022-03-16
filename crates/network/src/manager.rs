use enum_as_inner::EnumAsInner;
use crate::{client::{Client, PacketSender}, host::{Host, PacketReceiver}};

// Network manager
#[derive(EnumAsInner)]
pub enum NetworkManager {
    Host(Host),
    Client(Client),
}
/*
// A packet manager that can send/receive specific packets
#[derive(EnumAsInner)]
pub enum PacketManager<P: Payload> {
    Sender(PacketSender<P>),
    Receiver(PacketReceiver<P>)
}
*/