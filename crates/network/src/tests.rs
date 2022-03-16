#[cfg(test)]
mod tests {
    use std::net::{SocketAddrV6, Ipv6Addr};

    use serde::{Serialize, Deserialize};

    use crate::{client::{Client, PacketSender}, host::{Host, PacketReceiver}};

    #[test]
    fn test() {
        // Create a host, and open it's port on a specific port
        let host = Host::open("3333").unwrap();
        // Create a client and connect to a server
        let client = Client::connect(Ipv6Addr::LOCALHOST, host.address().port()).unwrap();

        #[derive(Serialize, Deserialize)]
        struct TestPayload {
            pub name: String,
            pub value: i32,
        }

        // Create a packet sender and a packet receiver
        let mut sender = PacketSender::<TestPayload>::new(&client, 0).unwrap();
        sender.send(TestPayload {
            name: "Jed le Jribi".to_string(),
            value: -5,
        });
        let receiver = PacketReceiver::<TestPayload>::new(&host, 0).unwrap();
        let payload = receiver.receive();

        dbg!(payload.name);
        dbg!(payload.value);
    }
}