use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use assets::AssetManager;
use debug::DefaultDebugRendererType;
use ecs::{ComponentManager, Entity};
use math::octrees::OctreeNode;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rendering::{Model, Shader, Texture};
use world_data::WorldData;

use crate::{chunk_data::ChunkCoords, mesher, ChunkData, VoxelGenerator};
use crate::{TModel, Voxel, MAIN_CHUNK_SIZE};

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
    // Are we currently waiting for the voxels to finish generating?
    pub voxels_generating: bool,
    // Camera location and forward vector
    pub camera_location: veclib::Vector3<f32>,
    pub camera_forward_vector: veclib::Vector3<f32>,
    // The chunk entityies that are waiting for the chunks to finish generating until they can mass-add to the entity manager
    pub pending_chunks: Vec<(ChunkCoords, TModel)>,
}

// Chunk manager. This is how each chunk entity is created
impl ChunkManager {
    // Are we allowed to update the octree?
    pub fn octree_update_valid(&self) -> bool {
        // Only update the octree if we don't have entities to remove and we don't have chunks to generate
        let entities_to_remove = self.entities_to_remove.len();
        let chunks_to_generate = self.chunks_to_generate.len();
        return entities_to_remove == 0 && chunks_to_generate == 0;
    }
    // Add a chunk
    pub fn add_chunk(&mut self, coords: ChunkCoords) {
        self.chunks_to_generate.push(coords);
    }
    // Remove a chunk
    pub fn remove_chunk(&mut self, coords: &ChunkCoords) -> Option<()> {
        if self.chunks.contains(&coords.center) {
            // Only remove the chunk if it exists in the first place
            self.chunks.remove(&coords.center);
            return Some(());
        } else {
            return None;
        }
    }
    // Add a chunk entity
    pub fn add_chunk_entity(&mut self, coords: &ChunkCoords, entity_id: usize) {
        self.entities.insert(coords.center, entity_id);
    }
    // Remove a chunk entity
    pub fn remove_chunk_entity(&mut self, coords: &ChunkCoords) {
        // Check if we even have the chunk entity in the first place
        let id = self.entities.remove(&coords.center).unwrap();
        self.entities_to_remove.insert(coords.center, id);
    }
    // The priority function
    pub fn priority_function(a: &ChunkCoords, camera_forward_vector: &veclib::Vector3<f32>, camera_position: &veclib::Vector3<f32>) -> f32 {
        let priority = camera_forward_vector.dot((*camera_position - veclib::Vector3::<f32>::from(a.center)).normalized());
        priority
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
                let ad = Self::priority_function(&a, &camera_forward_vector, &camera_position);
                let bd = Self::priority_function(&b, &camera_forward_vector, &camera_position);
                bd.partial_cmp(&ad).unwrap_or(Ordering::Equal)
            });
        }
        // Debug draw the chunks to generate
        for chunk_to_generate in self.chunks_to_generate.iter() {
            let t = DefaultDebugRendererType::CUBE(
                veclib::Vector3::from(chunk_to_generate.center),
                veclib::Vector3::new(chunk_to_generate.size as f32, chunk_to_generate.size as f32, chunk_to_generate.size as f32),
            );
        }

        // This chunk will always have a valid model and chunk data
        let mut final_chunk: Option<(ChunkData, TModel)> = None;
        let x = self.chunks_to_generate[0..(1.min(self.chunks_to_generate.len()))].get(0);
        match x {
            Some(coord) => {
                // Get the chunk coords
                let chunk_coords = coord.clone();

                // Decide between generating the chunk or start the generation of the voxel data
                if self.voxels_generating && (self.last_frame_voxels_generated + crate::FRAME_THRESHOLD) >= frame_count {
                    // The voxels are generating, so wait until we reached a satisfactory frame count
                    // We reached the limit, read the compute buffer
                    self.voxels_generating = false;
                    self.last_frame_voxels_generated = 0;
                    // Generate the data for this chunk
                    let (has_surface, voxels) = voxel_generator.generate_voxels_end(chunk_coords.size, chunk_coords.depth, chunk_coords.position);
                    // Since we just generated the chunk we can remove it from the generated chunks
                    self.chunks_to_generate.remove(0);

                    // If we don't have a surface, no need to create a model for this chunk
                    if has_surface {
                        // We have a surface, create the model
                        let coords = chunk_coords.clone();
                        let model = mesher::generate_model(&voxels, chunk_coords.size as usize, true, true);
                        let chunk_data = ChunkData { coords: coords, voxels: voxels };
                        final_chunk = Some((chunk_data, model));
                    }
                } else {
                    // The voxels didn't start generation yet, so start it
                    self.voxels_generating = true;
                    self.last_frame_voxels_generated = frame_count;
                    voxel_generator.generate_voxels_start(chunk_coords.size, chunk_coords.depth, chunk_coords.position);
                    // We aren't generating a mesh so return none
                }
            }
            None => {}
        }

        // The system was flawed...
        match final_chunk {
            Some((data, model)) => {
                self.chunks.insert(data.coords.center.clone());
                self.pending_chunks.push((data.coords, model));
            }
            None => {}
        }

        // Remove the entities after all the new ones got generated
        if self.chunks_to_generate.len() == 0 {
            let entities_to_remove = self.entities_to_remove.iter().map(|x| x.1.clone()).collect();
            self.entities_to_remove.clear();
            let yoinked = std::mem::replace(&mut self.pending_chunks, Vec::new());
            return Some((yoinked, entities_to_remove));
        }
        return None;
    }
}
