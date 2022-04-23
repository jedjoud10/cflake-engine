use crate::{PackedVoxelData, PersistentVoxelData, VoxelData};
use parking_lot::{Mutex, MutexGuard};
use std::{
    cell::{Ref, RefCell},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

// Contains a unique execution ID and buffer id
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VoxelDataBufferId {
    idx: usize,
}

// Voxel data that is generated for a single chunk
// This data can be sent to other threads for multi-threaded mesh generation
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
    // Load the underlying voxel data
    pub fn load(&self) -> MutexGuard<VoxelData> {
        self.data.lock()
    }
}

// Data that can be *shared* between threads
pub type SharedVoxelData = Arc<MutexVoxelData>;

// A buffer that contains multiple StoredVoxelDatas
// This will be contained within the main terrain generator
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
    pub fn store(&mut self, stored: &PackedVoxelData) -> (VoxelDataBufferId, PersistentVoxelData) {
        // Index
        let idx = self.find();

        // Store in the next available buffer
        let borrowed = self.buffer.borrow();
        let shared_voxel_data = borrowed.get(idx).unwrap();
        let mut data = shared_voxel_data.data.lock();
        let persistent = data.store(stored);
        (VoxelDataBufferId { idx }, persistent)
    }
    // Get a shared voxel data using it's ID
    pub fn get(&self, id: VoxelDataBufferId) -> Option<Ref<SharedVoxelData>> {
        // Make sure the index is valid
        if id.idx >= self.buffer.borrow().len() {
            return None;
        }

        // Ref::Map is a blessing
        Some(Ref::map(self.buffer.borrow(), |x| x.get(id.idx).unwrap()))
    }
    // Len
    pub fn len(&self) -> usize {
        self.buffer.borrow().len()
    }
}
