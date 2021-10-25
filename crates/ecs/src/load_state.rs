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
