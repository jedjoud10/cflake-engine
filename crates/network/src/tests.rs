#[cfg(test)]
mod tests {
    use std::net::{SocketAddrV6, Ipv6Addr};

    use crate::{client::{Client, PacketSender}, host::{Host, PacketReceiver}};

    #[test]
    fn test() {
        // Create a host, and open it's port on a specific port
        let host = Host::open("3333").unwrap();
        // Create a client and connect to a server
        let client = Client::connect(Ipv6Addr::LOCALHOST, host.address().port()).unwrap();

        // Create a packet sender and a packet receiver
        //let sender = PacketSender::<f32>::new(&client);
        //let receiver = PacketReceiver::<f32>::new(&host);

        
    }
}