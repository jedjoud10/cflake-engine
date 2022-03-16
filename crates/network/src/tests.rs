#[cfg(test)]
mod tests {
    use std::net::Ipv6Addr;

    use serde::{Deserialize, Serialize};

    use crate::{
        client::{Client, PacketSender},
        host::{Host, PacketReceiver},
    };

    #[test]
    fn test() {
        // Create a host, and open it's port on a random port
        let host = Host::open().unwrap();
        // Create a client and connect to a server
        let client = Client::connect(Ipv6Addr::LOCALHOST, host.address().port()).unwrap();

        #[derive(Serialize, Deserialize)]
        struct TestPayload {
            pub name: String,
            pub value: i32,
        }

        // Create a packet sender and a packet receiver
        let mut sender = PacketSender::<TestPayload>::new(&client, 0).unwrap();
        sender
            .send(TestPayload {
                name: "Jed le Jribi".to_string(),
                value: -5,
            })
            .unwrap();
        let receiver = PacketReceiver::<TestPayload>::new(&host, 0, 256).unwrap();
        let payload = receiver.receive().unwrap();

        dbg!(&payload[0].name);
        dbg!(&payload[0].value);
    }
}
