use crate::{mesher::{Mesher, GeneratedMeshSurface}, ChunkCoords, VoxelDataBuffer, VoxelDataBufferId};
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

// How the mesh scheduler should generate the chunks
pub struct MeshSchedulerSettings {
    // The number of threads that the mesh scheduler will use
    // If the value is None, the scheduler won't multithread mesh generation
    pub thread_num: Option<usize>,
}

// Chunk gen thread pool
struct MeshSchedulerThreadPool {
    pool: ThreadPool,
    // Comms
    sender: Sender<GenerationResult>,
    receiver: Receiver<GenerationResult>,
    mesh_tasks_running: RefCell<usize>,
}

// Mesh generation scheduler
pub struct MeshScheduler {
    // Pool
    pool: Option<MeshSchedulerThreadPool>,

    // Results
    cached: RefCell<Vec<GenerationResult>>,
}

impl MeshScheduler {
    // Create a new mesh scheduler
    pub fn new(settings: MeshSchedulerSettings) -> Self {
        let pool = settings.thread_num.map(|num| {
            assert!(num != 0, "Cannot have 0 mesher threads");
            // We must spawn the thread generation pool
            let (sender, receiver) = std::sync::mpsc::channel::<GenerationResult>();
            MeshSchedulerThreadPool {
                pool: ThreadPool::new(num),
                sender,
                receiver,
                mesh_tasks_running: RefCell::new(0),
            }
        });
        Self {
            pool,
            cached: RefCell::new(Vec::new()),
        }
    }
    // Start generating a mesh for the specific voxel data on another thread
    pub fn execute(&self, mesher: Mesher, buffer: &VoxelDataBuffer, id: VoxelDataBufferId) {
        if let Some(pool) = &self.pool {
            // Multithreaded
            // Lock it
            let data = buffer.get(id).clone();
            data.set_used(true);
            let sender = pool.sender.clone();
            *pool.mesh_tasks_running.borrow_mut() += 1;

            // Execute on a free thread
            pool.pool.execute(move || {
                // Generate the mesh
                let arc = data.as_ref();
                let unlocked = arc.load();
                let coords = mesher.coords;
                let (main, skirts, surface) = mesher.build(&unlocked);

                // Return
                sender.send(GenerationResult { coords, base: main, skirts, surface, id }).unwrap();
            });
        } else {
            // Singlethreaded
            let data = buffer.get(id).clone();
            data.set_used(true);

            // Generate the mesh
            let arc = data.as_ref();
            let unlocked = arc.load();
            let coords = mesher.coords;
            let (main, skirts, surface) = mesher.build(&unlocked);

            // Cached the result
            let mut cached = self.cached.borrow_mut();
            cached.push(GenerationResult { coords, base: main, skirts, surface, id });
        }
    }
    // Get the mesh results that were generated on other threads
    pub fn get_results(&self) -> Vec<GenerationResult> {
        self.pool
            .as_ref()
            .map(|pool| {
                // No need to cache the results since we can give them directly
                let results = pool.receiver.try_iter().collect::<Vec<_>>();
                *pool.mesh_tasks_running.borrow_mut() -= results.len();
                results
            })
            .unwrap_or_else(|| {
                // Poll first
                let mut results = self.cached.borrow_mut();
                std::mem::take(&mut results)
            })
    }
    // Get the amount of threads that are currently active
    pub fn active_mesh_tasks_count(&self) -> usize {
        self.pool.as_ref().map(|pool| *pool.mesh_tasks_running.borrow()).unwrap_or_default()
    }
}
