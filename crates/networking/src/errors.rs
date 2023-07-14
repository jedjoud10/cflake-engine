use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketSendError {
    #[error("{0}")]
    SocketError(std::io::Error),
}

#[derive(Error, Debug)]
pub enum PacketReceiveError {
    #[error("{0}")]
    SocketError(std::io::Error),
}
