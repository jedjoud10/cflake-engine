use crate::{PackedVoxelData, VoxelData};
use parking_lot::{Mutex, MutexGuard};
use std::{
    cell::{Ref, RefCell},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

// Can be sent to other threads
#[derive(Default)]
pub struct MutexVoxelData {
    data: Mutex<VoxelData>,
    used: AtomicBool,
}

impl MutexVoxelData {
    // Set the writing state of this mutex voxel data
    pub fn set_used(&self, locked: bool) {
        self.used.store(locked, Ordering::Relaxed);
    }
    // Load
    pub fn load(&self) -> MutexGuard<VoxelData> {
        self.data.lock()
    }
}

// Can be shared between thread
pub type SharedVoxelData = Arc<MutexVoxelData>;

// A buffer that contains multiple StoredVoxelDatas
pub struct VoxelDataBuffer {
    buffer: RefCell<Vec<SharedVoxelData>>,
}

impl Default for VoxelDataBuffer {
    fn default() -> Self {
        Self {
            buffer: RefCell::new(vec![SharedVoxelData::default()]),
        }
    }
}

impl VoxelDataBuffer {
    // Get the index of the next free voxel data
    fn find(&self) -> usize {
        let idx = self.buffer.borrow().iter().position(|shared| !shared.used.load(Ordering::Relaxed));
        match idx {
            Some(idx) => idx,
            None => {
                // If all the voxel datas are being used, add a new one and use it
                let mut borrowed = self.buffer.borrow_mut();
                borrowed.push(SharedVoxelData::default());
                borrowed.len() - 1
            }
        }
    }
    // Store some new voxel data
    pub fn store(&mut self, stored: &PackedVoxelData) -> usize {
        // Index
        let idx = self.find();

        // Store in the next available buffer
        let borrowed = self.buffer.borrow();
        let shared_voxel_data = borrowed.get(idx).unwrap();
        let mut data = shared_voxel_data.data.lock();
        data.store(stored);
        idx
    }
    // Get
    pub fn get(&self, idx: usize) -> Ref<SharedVoxelData> {
        Ref::map(self.buffer.borrow(), |x| x.get(idx).unwrap())
    }
    // Len
    pub fn len(&self) -> usize {
        self.buffer.borrow().len()
    }
}
