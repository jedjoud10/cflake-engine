use std::ffi::c_void;

// Raw pointer from a component vector
#[derive(Clone, Copy)]
pub struct StorageVecPtr(pub(crate) *mut c_void);