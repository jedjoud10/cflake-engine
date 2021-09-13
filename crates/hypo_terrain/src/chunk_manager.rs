use std::collections::{HashMap, HashSet};

use hypo_defaults::components;
use hypo_rendering::{Model, Shader};
use hypo_system_event_data::SystemEventData;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{chunk_data::ChunkCoords, mesher, ChunkData, VoxelGenerator, CHUNK_SIZE};

// Manages the chunks, makes it easier to do multithreading / compute shader stuff
#[derive(Default)]
pub struct ChunkManager {
    pub chunks_to_generate: Vec<ChunkCoords>,
    // Just the chunk data
    pub chunks: HashMap<veclib::Vector3<i64>, ChunkData>,
    pub entities: HashMap<veclib::Vector3<i64>, u16>,
    pub entities_to_remove: HashMap<veclib::Vector3<i64>, u16>,
    // The last frame chunk voxels where generated
    pub last_frame_voxels_generated: u64,
    // Are we currently waiting for the voxels to finish generating?
    pub voxels_generating: bool,
    // The parent nodes that got their children generated
    pub parent_children_generation_count: HashMap<veclib::Vector3<i64>, u8>,
}

// How many frames to wait before getting the data from the compute shader
pub const FRAMES_COMPUTE_DELAY: u64 = 1;

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
        self.entities_to_remove.insert(coords.center, id);
    }
    // The priority function
    pub fn priority_function(a: &ChunkCoords, camera_forward_vector: &veclib::Vector3<f32>, camera_position: &veclib::Vector3<f32>) -> f32 {
        let priority = camera_forward_vector.dot((*camera_position - veclib::Vector3::<f32>::from(a.center)).normalized());
        priority
    }
    // Update the chunk manager
    pub fn update(&mut self, voxel_generator: &VoxelGenerator, data: &mut SystemEventData) -> (Vec<(ChunkCoords, Model)>, Vec<u16>) {
        // Sort the chunks to generate
        let camera_entity = data.entity_manager.get_entity(&data.custom_data.main_camera_entity_id).unwrap();
        let camera_position = camera_entity.get_component::<components::Transform>(data.component_manager).unwrap().position;
        let camera_forward_vector = camera_entity.get_component::<components::Transform>(data.component_manager).unwrap().rotation.mul_point(veclib::Vector3::Z);
        if !self.voxels_generating {
            // Sort the added nodes using a priority system
            self.chunks_to_generate.sort_by(|a, b| { 
                // Get the dot product
                let ad = Self::priority_function(&a, &camera_forward_vector, &camera_position);
                let bd = Self::priority_function(&b, &camera_forward_vector, &camera_position);
                bd.partial_cmp(&ad).unwrap()
            });
        }
        // Generate the data for some chunks, then create their model
        let mut new_chunks: Vec<(ChunkCoords, Model)> = Vec::new();
        let coord = self.chunks_to_generate[0..(1.min(self.chunks_to_generate.len()))].get(0);
        let generated_chunk = match coord {
            Some(_) => {
                // Get the chunk coords 
                let chunk_coords = coord.unwrap().clone();
                let mut voxels: Box<[super::Voxel]> = Box::new([super::Voxel::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]);
                // Decide between generating the chunk or start the generation of the voxel data
                if self.voxels_generating {
                    // The voxels are generating, so wait until we reached a satisfactory frame count
                    if data.time_manager.frame_count > self.last_frame_voxels_generated + FRAMES_COMPUTE_DELAY {
                        // We reached the limit, read the compute buffer
                        self.voxels_generating = false;
                        self.last_frame_voxels_generated = 0;
                        // Generate the data for this chunk
                        let has_surface = voxel_generator.generate_voxels_end(data, &mut voxels);
                        // Since we just generated the chunk we can remove it from the generated chunks
                        self.chunks_to_generate.remove(0);

                        // We generated this chunk, so we can add one to the the early-deletion parent node child counter
                        if self.parent_children_generation_count.contains_key(&chunk_coords.parent_center) {
                            let t = self.parent_children_generation_count.get_mut(&chunk_coords.parent_center).unwrap();
                            *t += 1;
                        } else {
                            self.parent_children_generation_count.insert(chunk_coords.parent_center, 1);
                        }

                        // If we don't have a surface, no need to create a model for this chunk
                        match has_surface {
                            Some(_) => {
                                // We have a surface, create the model
                                let coords = chunk_coords.clone();
                                let model = mesher::generate_model(&voxels, chunk_coords.size as usize, true, true);
                                // Save the chunk's data, though don't save the mode
                                let chunk_data = ChunkData { coords: coords, voxels: voxels };                        
                                Some((chunk_data, model))
                            }
                            None => {
                                // We don't have a surface, no need to create the model, but rerun the update loop to find a model that doe have a surface
                                None
                            }
                        }
                    } else {
                        // Wait...
                        None
                    }
                } else {
                    // The voxels didn't start generation yet, so start it
                    self.voxels_generating = true;
                    self.last_frame_voxels_generated = data.time_manager.frame_count;
                    voxel_generator.generate_voxels_start(data, &chunk_coords.size, &chunk_coords.position);           
                    // We aren't generating a mesh so return none
                    None         
                }                
            }
            None => { /* No chunk to generate */ None }
        };        

        // Update the actual chunks in the main thread       
        match generated_chunk {
            Some((data, model)) => {
                // This chunk has a surface and a model, so add it to the world as a new entity
                let coords = data.coords.clone();                
                self.chunks.insert(coords.center.clone(), data);
                new_chunks.push((coords, model));
            }
            None => {}
        }
        let mut entities_to_remove: HashSet<u16> = HashSet::new();

        // Detect when one of the parent children nodes reaches 8 child nodes generated, that means we can delete it early
        let mut nodes_to_parents: Vec<veclib::Vector3<i64>> = Vec::new();
        for (octree_parent, count) in self.parent_children_generation_count.iter() {
            if *count == 8 {
                nodes_to_parents.push(octree_parent.clone());
            }
        }
        // Remove the nodes
        for node in nodes_to_parents.iter() {
            self.parent_children_generation_count.remove(node);
            // Remove the chunks early
            let id = self.entities_to_remove.get(node);
            match id {
                Some(id) => { 
                    entities_to_remove.insert(*id);
                    // Remove it from the cache
                    self.entities_to_remove.remove(node);
                }, 
                _ => {}
            };
        }
    
        // If the new chunks are 0, then we can delete all the old chunks
        if self.chunks_to_generate.len() == 0 {
            let a = self.entities_to_remove.values().map(|x| *x).collect::<Vec<u16>>();
            // Clear the removed entities
            entities_to_remove.extend(a);
            self.entities_to_remove.clear();
            self.parent_children_generation_count.clear();
        }

        return (new_chunks, entities_to_remove.iter().map(|x| *x).collect::<Vec<u16>>());
    }
}
