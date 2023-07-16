use std::sync::Arc;

use atomic_float::AtomicF32;

// F32 value that can be passed as parameter to sources
pub trait Value: Sync + Send {
    type Storage: Sync + Send;

    fn new_storage_from(self) -> Self::Storage;
    fn cache(storage: &mut Self::Storage);
    fn fetch(storage: &Self::Storage) -> f32;
}

impl Value for f32 {
    type Storage = f32;

    fn new_storage_from(self) -> Self::Storage {
        self
    }

    fn cache(_storage: &mut Self::Storage) {}

    fn fetch(storage: &Self::Storage) -> f32 {
        *storage
    }
}

impl Value for Arc<AtomicF32> {
    type Storage = (Arc<AtomicF32>, f32);

    fn new_storage_from(self) -> Self::Storage {
        (self, 0.0)
    }

    fn cache(storage: &mut Self::Storage) {
        let x = storage.0.load(std::sync::atomic::Ordering::Relaxed);
        storage.1 = x;
    }

    fn fetch(storage: &Self::Storage) -> f32 {
        storage.1
    }
}
