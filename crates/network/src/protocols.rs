use std::{io::{Error, Read, ErrorKind, Cursor, BufReader}, net::UdpSocket};

use crate::{Payload, manager::NetworkManager, cache::NetworkCache, PacketMetadata, serialize_payload};
mod endpoint;
use self::{udp::{UdpPacketReceiver, UdpPacketSender}, transport::{PacketChannel, PacketDirection}};
pub use endpoint::*;
pub mod tcp;
pub mod transport;
pub mod udp;

// Poller
pub trait Poller {
    type Socket;

    // Poll any new received payloads and put them in their respective bucket
    fn poll_from(cache: &mut NetworkCache, socket: &mut Self::Socket) -> Result<(), Error>;
}

// Protocol trait that will be implemented on 
pub trait Protocol<P: Payload> where Self: Sized {
    type Sender;
    type Receiver;
    // Create a channel
    fn new(manager: &mut NetworkManager, direction: PacketDirection) -> Result<PacketChannel<P, Self>, Error>;
    

    // Read from the receiver
    fn recv(receiver: &mut Self::Receiver, cache: &NetworkCache) -> Option<Vec<P>>;
    // Send using the sender
    fn send(sender: &mut Self::Sender, payload: P) -> Result<usize, Error>;
}
// Udp protocol
pub struct UdpProtocol;

impl Poller for UdpProtocol {
    type Socket = UdpSocket;

    fn poll_from(cache: &mut NetworkCache, socket: &mut Self::Socket) -> Result<(), Error> {
        // Read until the buffer generates an error
        loop {
            // Simple buffer
            let mut buf = vec![0; cache.max_buffer_size];
            let (len, sender) = match socket.recv_from(&mut buf) {
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

            // Get the ID
            let cursor = Cursor::new(&buf);
            let mut reader = BufReader::new(cursor);
            let mut metadata = [0u8; 8];
            reader.read_exact(&mut metadata)?;
            let metadata = PacketMetadata { bucket_id: u64::from_be_bytes(metadata) };
            let mut payload = Vec::default();
            reader.read_to_end(&mut payload)?;

            // Sort into the respective bucket
            cache.push(metadata, payload);
        }
        Ok(())
    }
}

impl<P: Payload + 'static> Protocol<P> for UdpProtocol {
    type Sender = UdpPacketSender<P>;
    type Receiver = UdpPacketReceiver<P>;

    fn new(manager: &mut NetworkManager, direction: PacketDirection) -> Result<PacketChannel<P, Self>, Error> {
        todo!()
    }    

    // Read the cached up payloads from the receiver
    fn recv(receiver: &mut Self::Receiver, cache: &NetworkCache) -> Option<Vec<P>> {
        let id = receiver.bucket_id;
        let meta = PacketMetadata { bucket_id: id };
        let payload_bytes_bucket = cache.drain_bucket(meta)?;
        // Deserialize all the payloads now
        let mut deserialized = Vec::<P>::new();
        for payload_bytes in payload_bytes_bucket {
            let payload = serde_json::from_slice::<P>(&payload_bytes).ok()?;
            deserialized.push(payload);
        }
        Some(deserialized)
    }

    // Send a payload
    fn send(sender: &mut Self::Sender, payload: P) -> Result<usize, Error> {
        // Serialize the data
        let bytes = serialize_payload::<P>(PacketMetadata { bucket_id: sender.bucket_id }, payload)?;
        let len = sender.socket.send(&bytes)?;
        println!("Sent '{}' bytes!", len);
        Ok(len)
    }
}


// Tcp protocol
pub struct TcpProtocol;
