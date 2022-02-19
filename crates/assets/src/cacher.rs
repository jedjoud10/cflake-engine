use crate::metadata::AssetMetadata;
use ahash::AHashMap;

// Cacher that keeps assets loaded in, so it's cheaper to load them later
#[derive(Default)]
pub struct AssetCacher {
    cached: AHashMap<AssetMetadata, Vec<u8>>,
}

impl AssetCacher {
    // Cache an asset for later
    pub(crate) fn cache(&mut self, meta: AssetMetadata, bytes: Vec<u8>) {
        self.cached.insert(meta, bytes);
    }
    // Try to load a cached asset
    pub(crate) fn try_load(&self, meta: &AssetMetadata) -> Option<&[u8]> {
        self.cached.get(meta).map(|x| x.as_slice())
    }
}
