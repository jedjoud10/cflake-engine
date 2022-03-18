use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{io::{self, BufRead, BufReader, Cursor, Read}, collections::HashMap, cell::RefCell};


// Stored network cache
#[derive(Default)]
pub struct NetworkCache {
    buckets: HashMap<PacketMetadata, RefCell<Vec<Vec<u8>>>>,
}

impl NetworkCache {
    pub fn new(max_buffer_size: usize) -> Self {
        Self {
            buckets: Default::default(),
        }
    }
    // Clear all cache
    pub fn clear(&mut self) {
        for (_, slots) in self.buckets.iter() {
            let mut borrow = slots.borrow_mut();
            borrow.clear();
        }
    }
    // Drain a whole bucket of payloads
    pub fn drain_bucket(&self, meta: PacketMetadata) -> Option<Vec<Vec<u8>>> {
        let vec = self.buckets.get(&meta)?;
        let mut borrowed = vec.borrow_mut();
        let stolen = std::mem::take(&mut *borrowed);
        if stolen.is_empty() {
            None
        } else {
            Some(stolen)
        }
    }
    // Push some received payload data into the corresponding slot
    pub fn push(&mut self, meta: PacketMetadata, data: Vec<u8>) {

    }
}


pub trait Payload: Serialize + DeserializeOwned {}
impl<T> Payload for T where T: Serialize + DeserializeOwned {}
// Packet metadata that contains some info on how we should treat the incoming payload
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct PacketMetadata {
    // Payload type
    pub bucket_id: u64,
}

impl From<u64> for PacketMetadata {
    fn from(bucket_id: u64) -> Self {
        PacketMetadata {
            bucket_id,
        }
    }
}

// Serialize a payload, with it's packet metadata
pub fn serialize_payload<P: Payload>(meta: PacketMetadata, payload: P) -> Result<Vec<u8>, io::Error> {
    // Serialze the metadata
    let meta = meta.bucket_id.to_be_bytes();
    // Serialize the payload
    let payload = serde_json::to_string_pretty(&payload)?;

    println!("{:?}", &meta);
    println!("{}", &payload);

    // Convert to bytes
    let mut meta = meta.to_vec();
    let payload = payload.into_bytes();

    // Extend
    meta.extend(payload);
    Ok(meta)
}

/*
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

    // Create a new channel
    fn new(manager: &mut NetworkManager, direction: PacketDirection, bucket_id: u64) -> Result<PacketChannel<P, Self>, Error> {
        Ok(match (manager, direction) {
            (NetworkManager::Host(host), PacketDirection::ClientToServer) => {
                // Create a new receiver on server
                PacketChannel::Receiver(UdpPacketReceiver::<P>::new(bucket_id))
            },
            (NetworkManager::Host(host), PacketDirection::ServerToClient(connected)) => {
                // Create a new sender on the server
                todo!()
            },
            (NetworkManager::Client(client), PacketDirection::ClientToServer) => {
                // Create a new sender on the client
                PacketChannel::Sender(UdpPacketSender::<P>::new(bucket_id, client.outbound_socket().udp_stream.try_clone()?))
            },
            (NetworkManager::Client(client), PacketDirection::ServerToClient(connected)) => {
                /*
                // Create a new receiver on the client, but only if we are the specified one
                if client.connected_id() == connected {
                    PacketChannel::Receiver(UdpPacketReceiver::<P>::new(bucket_id))
                } else { PacketChannel::None }
                */
                todo!()
            },
        })
    }    

    // Read the cached up payloads from the receiver
    fn recv(receiver: &mut Self::Receiver, cache: &NetworkCache) -> Option<Vec<P>> {
        let id = receiver.bucket_id();
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
        let bytes = serialize_payload::<P>(PacketMetadata { bucket_id: sender.bucket_id() }, payload)?;
        let len = sender.socket().send(&bytes)?;
        println!("Sent '{}' bytes!", len);
        Ok(len)
    }
}
*/
