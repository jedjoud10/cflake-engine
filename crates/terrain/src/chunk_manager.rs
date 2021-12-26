use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::thread;

use crate::TModel;
use crate::{chunk_data::ChunkCoords, mesher, VoxelGenerator};

// Manages the chunks, makes it easier to do multithreading / compute shader stuff
#[derive(Default)]
pub struct ChunkManager {
    pub chunks_to_generate: Vec<ChunkCoords>,
    // Just the chunk data
    pub chunks: HashSet<veclib::Vector3<i64>>,
    pub entities: HashMap<veclib::Vector3<i64>, usize>,
    pub entities_to_remove: HashMap<veclib::Vector3<i64>, usize>,
    // The last frame chunk voxels where generated
    pub last_frame_voxels_generated: u64,
    pub voxels_generating: bool,
    // Camera location and forward vector
    pub camera_location: veclib::Vector3<f32>,
    pub camera_forward_vector: veclib::Vector3<f32>,
}

// Chunk manager. This is how each chunk entity is created
impl ChunkManager {
    // Are we allowed to update the octree?
    pub fn octree_update_valid(&self) -> bool {
        // Only update the octree if we don't have entities to remove and we don't have chunks to generate
        let entities_to_remove = self.entities_to_remove.len();
        let chunks_to_generate = self.chunks_to_generate.len();
        entities_to_remove == 0 && chunks_to_generate == 0
    }
    // Add a chunk
    pub fn add_chunk(&mut self, coords: ChunkCoords) {
        self.chunks_to_generate.push(coords);
    }
    // Remove a chunk
    pub fn remove_chunk(&mut self, coords: &ChunkCoords) {
        if self.chunks.contains(&coords.center) {
            // Only remove the chunk if it exists in the first place
            self.chunks.remove(&coords.center);
            // Check if we even have the chunk entity in the first place
            let id = self.entities.remove(&coords.center).unwrap();
            self.entities_to_remove.insert(coords.center, id);
        }
    }
    // Add a chunk entity
    pub fn add_chunk_entity(&mut self, coords: &ChunkCoords, entity_id: usize) {
        self.entities.insert(coords.center, entity_id);
    }
    // The priority function
    pub fn priority_function(a: &ChunkCoords, camera_forward_vector: &veclib::Vector3<f32>, camera_position: &veclib::Vector3<f32>) -> f32 {
        camera_forward_vector.dot((*camera_position - veclib::Vector3::<f32>::from(a.center)).normalized())
    }
    // Update the location and forward vector of the camera entity
    pub fn update_camera_view(&mut self, position: veclib::Vector3<f32>, forward_vector: veclib::Vector3<f32>) {
        self.camera_location = position;
        self.camera_forward_vector = forward_vector;
    }
    // Update the chunk manager
    pub fn update(&mut self, voxel_generator: &mut VoxelGenerator, frame_count: u64) -> Option<(Vec<(ChunkCoords, TModel)>, Vec<usize>)> {
        // Sort the chunks to generate
        
        if !self.voxels_generating {
            // Sort the added nodes using a priority system
            let camera_position = self.camera_location;
            let camera_forward_vector = self.camera_forward_vector;
            self.chunks_to_generate.sort_by(|a, b| {
                // Get the dot product
                let ad = Self::priority_function(a, &camera_forward_vector, &camera_position);
                let bd = Self::priority_function(b, &camera_forward_vector, &camera_position);
                bd.partial_cmp(&ad).unwrap_or(Ordering::Equal)
            });
        }
        // This chunk will always have a valid model and chunk data
        let mut final_chunk: Option<TModel> = None;
        let x = self.chunks_to_generate.get(0);
        match x {
            Some(coord) => {
                // Get the chunk coords
                let chunk_coords = coord.clone();

                // Decide between generating the chunk or start the generation of the voxel data
                if self.voxels_generating && (self.last_frame_voxels_generated + crate::FRAME_THRESHOLD) < frame_count {
                    // The voxels are generating, so wait until we reached a satisfactory frame count
                    // We reached the limit, read the compute buffer
                    self.voxels_generating = false;
                    self.last_frame_voxels_generated = 0;

                    // If we don't have a surface, no need to create a model for this chunk
                    match voxel_generator.generate_voxels_end(chunk_coords.size, chunk_coords.depth, chunk_coords.position) {
                        Some(voxels) => {
                            // We have a surface, create the model
                            // Valid TModel
                            let tmodel = mesher::generate_model(&voxels, chunk_coords, true);
                            final_chunk = Some(tmodel);
                        }
                        None => {
                            /* We don't have a surface */                            
                        }
                    }
                    self.chunks_to_generate.remove(0);
                } else {
                    // Uh oh
                    if !self.voxels_generating {
                        // The voxels didn't start generation yet, so start it
                        voxel_generator.generate_voxels_start(chunk_coords.size, chunk_coords.depth, chunk_coords.position);
                        self.voxels_generating = true;
                        self.last_frame_voxels_generated = frame_count;
                    }
                }
            }
            None => {}
        }
        // Simply add the new generated chunk, and remove the chunks that must be removed
        let new_chunk: Vec<(ChunkCoords, TModel)> = match final_chunk {
            Some(x) => vec![(x.coords.clone(), x)],
            None => Vec::new(),
        };
        let entities_to_remove = self.entities_to_remove.iter().map(|x| *x.1).collect();  
        Some((new_chunk, entities_to_remove))
    }
}
