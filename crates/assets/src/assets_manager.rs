use crate::{AssetCacher, ObjectCacher};

// Asset manager
#[derive(Default)]
pub struct AssetManager {
    // Asset cacher
    pub asset_cacher: AssetCacher,
    // Object cacher
    pub object_cacher: ObjectCacher
}

