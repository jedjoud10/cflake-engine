use std::sync::{Arc, atomic::Ordering};

use atomic_float::AtomicF32;
use atomic_traits::Atomic;
use parking_lot::RwLock;

// Any value that can be passed as parameter to sources
// Values can be updated after they have been executed and their change will be refelcted within the sources
pub trait Value<T>: Sync + Send {
    type Storage: Sync + Send;

    fn new_storage_from(self) -> Self::Storage;
    fn cache(storage: &mut Self::Storage);
    fn fetch(storage: &Self::Storage) -> T;
}

macro_rules! impl_basic {
    ($val:ty) => {
        impl Value<$val> for $val {
            type Storage = $val;
        
            fn new_storage_from(self) -> Self::Storage {
                self
            }
        
            fn cache(_storage: &mut Self::Storage) {}
        
            fn fetch(storage: &Self::Storage) -> $val {
                *storage
            }
        }
    };
}

macro_rules! impl_atomic {
    ($val:ty, $atomic:ty) => {
        impl Value<$val> for Arc<$atomic> {
            type Storage = (Arc<$atomic>, $val);
        
            fn new_storage_from(self) -> Self::Storage {
                (self, <$val>::default())
            }
        
            fn cache(storage: &mut Self::Storage) {
                storage.1 = storage.0.load(Ordering::Relaxed);
            }
        
            fn fetch(storage: &Self::Storage) -> $val {
                storage.1
            }
        }
    };
}

macro_rules! impl_rw_lock {
    ($val:ty) => {
        impl Value<$val> for Arc<RwLock<$val>> {
            type Storage = (Arc<RwLock<$val>>, $val);
        
            fn new_storage_from(self) -> Self::Storage {
                (self, <$val>::default())
            }
        
            fn cache(storage: &mut Self::Storage) {
                storage.1 = *storage.0.read();
            }
        
            fn fetch(storage: &Self::Storage) -> $val {
                storage.1
            }
        }
    };
}

impl_basic!(f32);
impl_basic!(vek::Vec3<f32>);

impl_atomic!(f32, AtomicF32);

impl_rw_lock!(vek::Vec3<f32>);
