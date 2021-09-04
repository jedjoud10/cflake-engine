use std::collections::HashMap;

use hypo_rendering::Model;

use crate::{ChunkData, VoxelGenerator, chunk_data::ChunkCoords, mesher};

// Manages the chunks, makes it easier to do multithreading / compute shader stuff
#[derive(Default)]
pub struct ChunkManager {
    pub chunks_to_generate: Vec<ChunkData>,
    // Just the chunk data
    pub chunks: HashMap<veclib::Vector3<i64>, ChunkData>,
    pub entities: HashMap<veclib::Vector3<i64>, u16>,
    pub entities_to_remove: Vec<u16>,
}

// How many chunks to generate per frame
pub const CHUNK_GENERATIONS_PER_FRAME: usize = 4;

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
    pub fn remove_chunk(&mut self, coords: &ChunkCoords) -> Option<()> {
        println!("Remove chunk");
        if self.chunks.contains_key(&coords.center) {
            // Only remove the chunk if it exists in the first place
            self.chunks.remove(&coords.center);
            return Some(());
        } else {
            return None;
        }
    }
    // Add a chunk entity
    pub fn add_chunk_entity(&mut self, coords: &ChunkCoords, entity_id: u16) {
        self.entities.insert(coords.center, entity_id);
    }
    // Remove a chunk entity
    pub fn remove_chunk_entity(&mut self, coords: &ChunkCoords) {
        // Check if we even have the chunk entity in the first place
        let id = self.entities.remove(&coords.center).unwrap();
        self.entities_to_remove.push(id);
    }
    // Update the chunk manager
    pub fn update(&mut self, voxel_generator: &VoxelGenerator) -> (Vec<(ChunkCoords, Model)>, Vec<u16>) {
        // Generate the data for some chunks, then create their model
        let mut new_chunks: Vec<(ChunkCoords, Model)> = Vec::new();
        
        // The chunks that are removed
        for i in 0..(CHUNK_GENERATIONS_PER_FRAME.min(self.chunks_to_generate.len())) {
            // Get the first chunk in the list
            let mut chunk_data = self.chunks_to_generate.remove(0);
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
            new_chunks.push((coords, model));
        }
        println!("{}", self.chunks_to_generate.len());
        // If the new chunks are 0, then we can delete all the old chunks 
        let mut entities_to_remove: Vec<u16> = Vec::new();
        if self.chunks_to_generate.len() == 0 {
            entities_to_remove = self.entities_to_remove.clone();
            // Clear the removed entities
            self.entities_to_remove.clear();
        }
        return (new_chunks, entities_to_remove);
    }
}