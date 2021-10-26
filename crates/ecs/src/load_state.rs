// The load state of a specific entity
#[derive(Clone)]
pub enum LoadState {
    Loaded,
    Unloaded,
    Cached,
}

impl Default for LoadState {
    fn default() -> Self {
        Self::Loaded
    }
}

// The sole reason why we update the load state of the current entity
pub enum LoadStateUpdateReason {
    AddedEntity,
    ExtremeFrustumCulling,
    Region,
    Explicit,
}