use std::net::{UdpSocket, TcpListener, TcpStream};

// Readers
pub struct ListenerSockets {
    // UDP and TCP
    pub udp_listen: UdpSocket,
    pub tcp_listen: TcpListener,
}


// Writers
pub struct StreamSockets {
    // UDP and TCP
    pub udp_stream: UdpSocket,
    pub tcp_stream: TcpStream,
}