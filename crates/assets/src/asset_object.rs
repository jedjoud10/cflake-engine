use crate::{main, Asset, Object};
use std::sync::Arc;

// Cached asset object
pub struct CachedObject<T>
where
    T: Send + Sync,
{
    pub arc: Arc<T>,
}
