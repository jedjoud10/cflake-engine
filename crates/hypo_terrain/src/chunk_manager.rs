use std::collections::HashMap;

use hypo_rendering::Model;

use crate::{ChunkData, VoxelGenerator, chunk_data::ChunkCoords, mesher};

// Manages the chunks, makes it easier to do multithreading / compute shader stuff
#[derive(Default)]
pub struct ChunkManager {
    pub chunks_to_generate: Vec<ChunkData>,
    pub chunks: HashMap<veclib::Vector3<i64>, ChunkData>
}

// Chunk manager. This is how each chunk entity is created
// 1. Add the ChunkCoords to the chunk_to_generate list
// TODO: 1.5: Check if the chunk could exist in the first place (d = y - 5.0, and funny shit)
// 2. Generate the data for that model using a compute shader
// 2. Generate the model for that chunk in another frame
// 3. Get the 
impl ChunkManager {
    // Add a chunk
    pub fn add_chunk(&mut self, coords: ChunkCoords) {
        let chunk_data = ChunkData::new(coords.clone());
        self.chunks_to_generate.push(chunk_data);
        println!("Add chunk for generation");
    }
    // Remove a chunk
    pub fn remove_chunk(&mut self, coords: ChunkCoords) {
        println!("Remove chunk");
        if self.chunks.contains_key(&coords.center) {
            // Only remove the chunk if it exists in the first place
            self.chunks.remove(&coords.center);
        }
    }
    // Update the chunk manager
    pub fn update(&mut self, voxel_generator: &VoxelGenerator) -> Vec<(ChunkCoords, Model)> {
        // Generate the data for some chunks, then create their model
        let mut output_chunks: Vec<(ChunkCoords, Model)> = Vec::new();
        for i in 0..1 {
            // Get the first chunk in the list
            let mut chunk_data = self.chunks_to_generate.swap_remove(0);
            // Generate the data for this chunk
            let has_surface = voxel_generator.generate_voxels(chunk_data.coords.size, chunk_data.coords.position, &mut chunk_data.voxels);
            // If we don't have a surface, no need to create a model for this chunk
            match has_surface {
                Some(_) => {},
                None => { /* We don't have a surface, no need to create the model */ continue; },
            };
            // We have a surface, create the model
            let model = mesher::generate_model(&chunk_data.voxels, true);
            let coords = chunk_data.coords.clone();
            // Save the chunk's data, though don't save the model
            self.chunks.insert(chunk_data.coords.center, chunk_data);
            output_chunks.push((coords, model));
        }
        return output_chunks;
    }
}