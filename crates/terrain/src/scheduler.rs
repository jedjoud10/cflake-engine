use crate::{mesher::Mesher, ChunkCoords, SharedVoxelData, VoxelData, VoxelDataBuffer};
use rendering::basics::mesh::{GeometryBuilder, Mesh};
use std::{
    cell::{Cell, RefCell},
    sync::{
        atomic::AtomicBool,
        mpsc::{Receiver, Sender, SyncSender},
        Arc,
    },
    thread::JoinHandle,
};
use threadpool::ThreadPool;

// The result that is sent to the main thread after we generate a mesh on a worker thread
pub struct MeshGenResult {
    pub coords: ChunkCoords,
    pub builders: (GeometryBuilder, GeometryBuilder),
    pub buffer_index: usize,
}

// Mesh generation scheduler
pub struct MeshScheduler {
    // Thread pool that contains 3 threads dedicated for mesh generation
    pool: ThreadPool,
    // Results
    sender: Sender<MeshGenResult>,
    receiver: Receiver<MeshGenResult>,
    mesh_tasks_running: RefCell<usize>,
}

impl Default for MeshScheduler {
    fn default() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<MeshGenResult>();
        Self {
            pool: ThreadPool::new(2),
            sender,
            receiver,
            mesh_tasks_running: RefCell::new(0),
        }
    }
}

impl MeshScheduler {
    // Start generating a mesh for the specific voxel data on another thread
    pub fn execute(&self, mesher: Mesher, buffer: &VoxelDataBuffer, index: usize) {
        // Lock it
        let data = buffer.get(index).clone();
        data.set_used(true);
        let sender = self.sender.clone();
        *self.mesh_tasks_running.borrow_mut() += 1;

        // Execute on a free thread
        self.pool.execute(move || {
            // Generate the mesh
            let arc = data.as_ref();
            let unlocked = arc.load();
            let coords = mesher.coords;
            let builders = mesher.build(&unlocked);

            // Return
            sender
                .send(MeshGenResult {
                    coords,
                    builders,
                    buffer_index: index,
                })
                .unwrap();
        });
    }
    // Get the mesh results that were generated on other threads
    pub fn get_results(&self) -> Vec<MeshGenResult> {
        // Get all
        let results = self.receiver.try_iter().collect::<Vec<_>>();
        *self.mesh_tasks_running.borrow_mut() -= results.len();
        results
    }
    // Get the amount of threads that are currently active
    pub fn active_mesh_tasks_count(&self) -> usize {
        *self.mesh_tasks_running.borrow()
    }
}
