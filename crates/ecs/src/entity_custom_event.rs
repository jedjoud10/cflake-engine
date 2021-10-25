use crate::{LoadState, LoadStateUpdateReason};

// Entity custom event
pub enum EntityCustomEvent {
    LoadStateUpdate(LoadState, LoadStateUpdateReason)
}