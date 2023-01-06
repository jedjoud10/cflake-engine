use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorldBorrowError {
    #[error("Resource is not present in the world")]
    NotPresent,

    #[error("{0}")]
    BorrowError(core::cell::BorrowError),
}

#[derive(Error, Debug)]
pub enum WorldBorrowMutError {
    #[error("Resource is not present in the world")]
    NotPresent,

    #[error("{0}")]
    BorrowMutError(core::cell::BorrowMutError),
}