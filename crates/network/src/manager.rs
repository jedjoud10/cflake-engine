use crate::{client::Client, host::Host};
use enum_as_inner::EnumAsInner;

// Session
#[derive(EnumAsInner)]
pub enum Session {
    Multiplayer(NetworkManager),
    Singleplayer,
}

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
