use std::sync::mpsc::{Sender, Receiver, SyncSender};

use rendering::basics::mesh::Mesh;

use crate::{GlobalStoredVoxelData, ChunkCoords, mesher::Mesher};

// The result of a mesh generation task
struct MeshTaskResult {
    coords: ChunkCoords,
    mesh: Mesh,
}

// A mesh generation task
struct MeshTask {
    mesher: Mesher,
    data: GlobalStoredVoxelData,
}

// Mesh generation scheduler
pub struct MeshScheduler {
    // Create a specific thread for mesh generation
    join: std::thread::JoinHandle<()>,
    sender: SyncSender<MeshTask>,
    receiver: Receiver<MeshTaskResult>,
}

impl MeshScheduler {
    // Create a new scheduler, and spawn the mesh generation thread
    pub fn new() -> Self {
        // Create the sync channel
        let (task_tx, task_rx) = std::sync::mpsc::sync_channel::<MeshTask>(1);
        let (result_tx, result_rx) = std::sync::mpsc::sync_channel::<MeshTaskResult>(1);

        // Spawn the thread
        let join = std::thread::spawn(move || {
            // Wait until we receive new tasks
            loop {
                let task = task_rx.recv().unwrap();
                // Generate the mesh
                let mesher = task.mesher;
                let data = task.data;
                let mesh = mesher.build(data);
                // Send back the result
                result_tx.send(MeshTaskResult {
                    coords: mesher.coords,
                    mesh,
                }).unwrap();
            }
        });

        Self {
            join,
            sender: task_tx,
            receiver: result_rx,
        }
    }
    // Send a chunk to be generated
    // TODO: Parallelisation, and make a round robin buffer that contains multiple StoredVoxelDatas
    pub fn generate(&mut self, mesher: Mesher, data: &GlobalStoredVoxelData) {
        self.sender.send(MeshTask {
            data: data.clone(),
            mesher,
        }).unwrap();
    }
    // Try to 
}