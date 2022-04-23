use crate::{
    mesher::{GeneratedMeshSurface, Mesher},
    ChunkCoords, VoxelDataBuffer, VoxelDataBufferId,
};
use rendering::basics::mesh::GeometryBuilder;
use std::{
    cell::RefCell,
    sync::mpsc::{Receiver, Sender},
};
use threadpool::ThreadPool;

// The result that is sent to the main thread after we generate a mesh on a worker thread
pub struct GenerationResult {
    // The coordinates of the chunk
    pub coords: ChunkCoords,

    // The main geometry builder, generates most of the mesh
    pub base: GeometryBuilder,

    // The skirts' geometry builder, used only to hide seams between multiple chunks
    pub skirts: GeometryBuilder,

    // The mesh's surface
    pub surface: GeneratedMeshSurface,

    // The ID of the voxel data that we used to generate this mesh
    pub id: VoxelDataBufferId,
}

// Mesh generation scheduler
pub struct MeshScheduler {
    // Actual thread pool that contains the task threads
    pool: ThreadPool,

    // Given to the other threads to allow them to send the results back
    sender: Sender<GenerationResult>,

    // Always on the main thread, waiting for results
    receiver: Receiver<GenerationResult>,

    // Le number
    mesh_tasks_running: RefCell<usize>,
}

// Number of threads that will be allocated for mesh generation
const NUM_MESH_GEN_THREADS: usize = 2;

impl Default for MeshScheduler {
    fn default() -> Self {
        // Communication between threads
        let (sender, receiver) = std::sync::mpsc::channel::<GenerationResult>();
        Self {
            pool: ThreadPool::new(NUM_MESH_GEN_THREADS),
            sender,
            receiver,
            mesh_tasks_running: RefCell::new(0),
        }
    }
}

impl MeshScheduler {
    // Start generating a mesh for the specific voxel data on another thread
    pub fn execute(&self, mesher: Mesher, buffer: &VoxelDataBuffer, id: VoxelDataBufferId) {
        // Lock the data, since we will share it with another thread
        let data = buffer.get(id).unwrap().clone();
        data.set_used(true);

        // Clone the sender as well
        let sender = self.sender.clone();
        *self.mesh_tasks_running.borrow_mut() += 1;

        // Execute the mesher on a free thread
        self.pool.execute(move || {
            // Generate the mesh
            let coords = mesher.coords;
            let (main, skirts, surface) = mesher.build(&data.load());

            // Send the result back to the main thread
            sender
                .send(GenerationResult {
                    coords,
                    base: main,
                    skirts,
                    surface,
                    id,
                })
                .unwrap();
        });
    }
    // Get the mesh results that were generated on other threads
    pub fn get_results(&self) -> Vec<GenerationResult> {
        let res = self.receiver.try_iter().collect::<Vec<_>>();
        *self.mesh_tasks_running.borrow_mut() -= res.len();
        res
    }
    // Get the amount of threads that are currently active
    pub fn active_mesh_tasks_count(&self) -> usize {
        *self.mesh_tasks_running.borrow()
    }
}
