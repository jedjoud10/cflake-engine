#[cfg(test)]
mod tests {
    use std::net::{Ipv6Addr, SocketAddrV6};

    use serde::{Deserialize, Serialize};

    use crate::{
        client::Client,
        host::Host,
        manager::NetworkManager,
        protocols::{UdpProtocol, Protocol, transport::{PacketDirection, PacketChannel}, Poller
        }, PacketMetadata,
    };

    #[test]
    fn test() {
        // Create a host, and open it's port on a random port
        let host = Host::open(512, Some(5000)).unwrap();
        // Create a client and connect to a server
        let addr = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 5000, 0, 0);
        let client = Client::connect(addr, 512).unwrap();
        
        
        // Make two network managers
        let mut host = NetworkManager::Host(host);
        let mut client = NetworkManager::Client(client);

        // Params
        let meta = PacketMetadata { bucket_id: 0 };
        
        let mut udp: PacketChannel<f32, UdpProtocol> = UdpProtocol::new(&mut host, PacketDirection::ClientToServer).unwrap();
        let receiver = udp.as_receiver_mut().unwrap();

        let host = host.as_host_mut().unwrap();

        UdpProtocol::poll_from(host.cache_mut(), todo!()).unwrap();
        /*

        UdpProtocol::recv(receiver);
        */
        /*

        #[derive(Serialize, Deserialize)]
        struct CustomPayload {
            value: f32,
        }

        // Make a packet channel
        let mut sender = channel::<CustomPayload>(&mut client, &params, PacketDirection::ClientToServer).unwrap();
        let sender = sender.as_sender_mut().unwrap();
        let mut receiver = channel::<CustomPayload>(&mut host, &params, PacketDirection::ClientToServer).unwrap();
        let receiver = receiver.as_receiver_mut().unwrap();

        sender.send(CustomPayload { value: 0.591 }).unwrap();
        let payloads = receiver.recv().unwrap();
        dbg!(payloads[0].value);
        */
    }
}
