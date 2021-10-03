use std::collections::{HashMap, HashSet};

use debug::DefaultDebugRendererType;
use ecs::{ComponentManager, Entity};
use math::octree::OctreeNode;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rendering::{Model, Shader};
use system_event_data::SystemEventData;

use crate::CHUNK_SIZE;
use crate::{chunk_data::ChunkCoords, mesher, ChunkData, VoxelGenerator};

// Manages the chunks, makes it easier to do multithreading / compute shader stuff
#[derive(Default)]
pub struct ChunkManager {
    pub chunks_to_generate: Vec<ChunkCoords>,
    // Just the chunk data
    pub chunks: HashSet<veclib::Vector3<i64>>,
    pub entities: HashMap<veclib::Vector3<i64>, u16>,
    pub entities_to_remove: HashMap<veclib::Vector3<i64>, u16>,
    // The last frame chunk voxels where generated
    pub last_frame_voxels_generated: u64,
    // Are we currently waiting for the voxels to finish generating?
    pub voxels_generating: bool,
    // Camera location and forward vector
    pub camera_location: veclib::Vector3<f32>,
    pub camera_forward_vector: veclib::Vector3<f32>,
}

// Chunk manager. This is how each chunk entity is created
// 1. Add the ChunkCoords to the chunk_to_generate list
// TODO: 1.5: Check if the chunk could exist in the first place (d = y - 5.0, and funny shit)
// 2. Generate the data for that model using a compute shader
// 2. Generate the model for that chunk in another frame
// 3. Get the
impl ChunkManager {
    // Are we allowed to update the octree?
    pub fn octree_update_valid(&self) -> bool {
        // Only update the octree if we don't have entities to remove and we don't have chunks to generate
        let entities_to_remove = self.entities_to_remove.len();
        let chunks_to_generate = self.chunks_to_generate.len();
        return (entities_to_remove == 0 && chunks_to_generate == 0);
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
    pub fn add_chunk_entity(&mut self, coords: &ChunkCoords, entity_id: u16) {
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
    pub fn update(&mut self, voxel_generator: &VoxelGenerator, compute_shader: &mut Shader, frame_count: u64) -> (Vec<(ChunkCoords, Model)>, Vec<u16>) {
        // Check if we are currently generating the chunks
        if self.chunks_to_generate.len() > 0 {
            // We are generating
        } else {
            // We are idle
        }

        // Sort the chunks to generate
        if !self.voxels_generating {
            // Sort the added nodes using a priority system
            let camera_position = self.camera_location;
            let camera_forward_vector = self.camera_forward_vector;
            self.chunks_to_generate.sort_by(|a, b| {
                // Get the dot product
                let ad = Self::priority_function(&a, &camera_forward_vector, &camera_position);
                let bd = Self::priority_function(&b, &camera_forward_vector, &camera_position);
                bd.partial_cmp(&ad).unwrap()
            });
        }
        // Debug draw the chunks to generate
        for chunk_to_generate in self.chunks_to_generate.iter() {
            let t = DefaultDebugRendererType::CUBE(
                veclib::Vector3::from(chunk_to_generate.center),
                veclib::Vector3::new(chunk_to_generate.size as f32, chunk_to_generate.size as f32, chunk_to_generate.size as f32),
            );
        }
        // Generate the data for some chunks, then create their model
        let mut new_chunks: Vec<(ChunkCoords, Model)> = Vec::new();

        // This chunk will always have a valid model and chunk data
        let mut final_chunk: Option<(ChunkData, Model)> = None;
        match self.chunks_to_generate[0..(1.min(self.chunks_to_generate.len()))].get(0) {
            Some(coord) => {
                // Get the chunk coords
                let chunk_coords = coord.clone();
                let mut voxels: Box<[super::Voxel]> = Box::new([super::Voxel::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]);

                // Decide between generating the chunk or start the generation of the voxel data
                if self.voxels_generating {
                    // The voxels are generating, so wait until we reached a satisfactory frame count
                    // We reached the limit, read the compute buffer
                    self.voxels_generating = false;
                    self.last_frame_voxels_generated = 0;
                    // Generate the data for this chunk
                    let has_surface = voxel_generator.generate_voxels_end(compute_shader, &mut voxels);
                    // Since we just generated the chunk we can remove it from the generated chunks
                    self.chunks_to_generate.remove(0);

                    // If we don't have a surface, no need to create a model for this chunk
                    match has_surface {
                        Some(_) => {
                            // We have a surface, create the model
                            let coords = chunk_coords.clone();
                            let model = mesher::generate_model(&voxels, chunk_coords.size as usize, true, true);
                            // Save the chunk's data, though don't save the mode
                            let chunk_data = ChunkData { coords: coords, voxels: voxels };
                            final_chunk = Some((chunk_data, model));
                        }
                        None => {
                            // We don't have a surface, no need to create the model, but rerun the update loop to find a model that doe have a surface
                        }
                    }
                } else {
                    // The voxels didn't start generation yet, so start it
                    self.voxels_generating = true;
                    self.last_frame_voxels_generated = frame_count;
                    voxel_generator.generate_voxels_start(compute_shader, &chunk_coords.size, &chunk_coords.position);
                    // We aren't generating a mesh so return none
                }
            }
            None => {}
        }

        let mut entities_to_remove: Vec<u16> = Vec::new();

        // The system was flawed...
        match final_chunk {
            Some((data, model)) => {
                self.chunks.insert(data.coords.center.clone());
                new_chunks.push((data.coords, model.clone()));
            }
            None => {}
        }

        // Remove the entities after all the new ones got generated
        if self.chunks_to_generate.len() == 0 {
            entities_to_remove = self.entities_to_remove.iter().map(|x| x.1.clone()).collect();
            self.entities_to_remove.clear();
        }
        let output = (new_chunks, entities_to_remove);
        return output;
    }
}
