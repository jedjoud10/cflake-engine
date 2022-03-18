#[cfg(test)]
mod tests {
    use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};

    use crate::{Client, Host};

    #[test]
    fn test() {
        // Host
        let mut host = Host::host().unwrap();
        // Client
        let client = Client::connect(host.local_addr()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));
        for x in 0..10 {
            host.poll().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        //client.send();

        for x in 0..10 {
            host.poll().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }

        /*
        use laminar::{Socket, Packet};

        // Creates the socket
        let mut socket1 = Socket::bind_any().unwrap();
        let packet_sender = socket1.get_packet_sender();
        let mut socket2 = Socket::bind_any().unwrap();
        let addr = socket2.local_addr().unwrap();
        let event_receiver = socket2.get_event_receiver();
        let packet_sender2 = socket2.get_packet_sender();

        // Bytes to sent
        let bytes = vec![0, 2, 0, 0];

        // Creates packets with different reliabilities
        let reliable = Packet::reliable_unordered(addr, bytes);
        packet_sender.send(reliable).unwrap();

        // Starts the socket, which will start a poll mechanism to receive and send messages.
        let _thread = std::thread::spawn(move || socket1.start_polling());
        let _thread2 = std::thread::spawn(move || socket2.start_polling());

        for event in event_receiver {
            match event {
                laminar::SocketEvent::Packet(packet) => {
                    let endpoint: SocketAddr = packet.addr();
                    let received_data: &[u8] = packet.payload();
                    packet_sender2.send(Packet::reliable_unordered(addr, vec![0])).unwrap();
                    dbg!(received_data);
                },
                laminar::SocketEvent::Connect(_) => todo!(),
                laminar::SocketEvent::Timeout(_) => todo!(),
                laminar::SocketEvent::Disconnect(_) => todo!(),
            }
        }
        */
        /*


        // Specifies on which stream and how to order our packets, check out our book and documentation for more information
        let unreliable = Packet::unreliable_sequenced(addr, bytes, Some(1));
        let reliable_sequenced = Packet::reliable_sequenced(addr, bytes, Some(2));
        let reliable_ordered = Packet::reliable_ordered(addr, bytes, Some(3));

        // Sends the created packets
        packet_sender.send(unreliable).unwrap();
        packet_sender.send(reliable).unwrap();
        packet_sender.send(unreliable_sequenced).unwrap();
        packet_sender.send(reliable_sequenced).unwrap();
        packet_sender.send(reliable_ordered).unwrap();
        */
    }
}
