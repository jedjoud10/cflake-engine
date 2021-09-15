use std::collections::{HashMap, HashSet};

use hypo_debug::DefaultDebugRendererType;
use hypo_defaults::components;
use hypo_ecs::{ComponentManager, Entity};
use hypo_rendering::{Model, Shader};
use hypo_system_event_data::SystemEventData;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{chunk_data::ChunkCoords, mesher, ChunkData, VoxelGenerator, CHUNK_SIZE};

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
    // The chunks that we need to add to the world and their corresponding parent node
    pub parent_children_added_entity_chunks: HashMap<veclib::Vector3<i64>, Vec<Option<(ChunkCoords, Model)>>>,
    // Camera location and forward vector
    pub camera_location: veclib::Vector3<f32>, pub camera_forward_vector: veclib::Vector3<f32>
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
    pub fn update_camera_view(&mut self, camera_entity: &Entity, component_manager: &ComponentManager) {
        self.camera_location = camera_entity.get_component::<components::Transform>(component_manager).unwrap().position;
        self.camera_forward_vector = camera_entity.get_component::<components::Transform>(component_manager).unwrap().rotation.mul_point(veclib::Vector3::Z);
    }
    // Update the chunk manager
    pub fn update(&mut self, voxel_generator: &VoxelGenerator, data: &mut SystemEventData, parent_child_count: HashMap<veclib::Vector3<i64>, u8>) -> (Vec<(ChunkCoords, Model)>, Vec<u16>) {
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
            let t = DefaultDebugRendererType::CUBE(veclib::Vector3::from(chunk_to_generate.center), veclib::Vector3::new(chunk_to_generate.size as f32, chunk_to_generate.size as f32, chunk_to_generate.size as f32));
            data.debug.debug_default(t, veclib::Vector3::ONE, false);
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

                        // Make sure the key exists
                        if !self.parent_children_added_entity_chunks.contains_key(&chunk_coords.parent_center) {
                            self.parent_children_added_entity_chunks.insert(chunk_coords.parent_center, Vec::new());
                        }

                        // If we don't have a surface, no need to create a model for this chunk
                        match has_surface {
                            Some(_) => {
                                // We have a surface, create the model
                                let coords = chunk_coords.clone();
                                let model = mesher::generate_model(&voxels, chunk_coords.size as usize, true, true);
                                // Save the chunk's data, though don't save the mode
                                let chunk_data = ChunkData { coords: coords, voxels: voxels };

                                // We are 100% sure that this key exists 
                                let c = self.parent_children_added_entity_chunks.get_mut(&chunk_coords.parent_center).unwrap();
                                c.push(Some((chunk_coords, model.clone())));                    
                                
                                Some((chunk_data, model))
                            }
                            None => {
                                // We don't have a surface, no need to create the model, but rerun the update loop to find a model that doe have a surface
                                let c = self.parent_children_added_entity_chunks.get_mut(&chunk_coords.parent_center).unwrap();
                                c.push(None);     
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
        let mut entities_to_remove: HashSet<u16> = HashSet::new();  
        let mut parents_removed: HashSet<veclib::Vector3<i64>> = HashSet::new();
        // Check all the parents and check if their chunk child count is 8
        for (parent_node, children) in self.parent_children_added_entity_chunks.iter() {
            // Check if it is equal the limit
            let limit = parent_child_count.get(parent_node).unwrap();
            if children.len() as u8 == *limit {
                // We have the correct amount of children, we can swap the children with their parent
                for child in children {
                    match child {
                        Some((coords, model)) => {
                            // Valid model
                            // TODO: Gotta make this not clone the values
                            new_chunks.push((coords.clone(), model.clone()));
                            self.chunks.insert(coords.center);
                        }
                        None => { /* No need */ }
                    }
                }
                parents_removed.insert(*parent_node);
                // We can remove the parent
                let entity_id = self.entities_to_remove.get(parent_node);
                match entity_id {
                    Some(parent_id) => {
                        entities_to_remove.insert(*parent_id);
                        self.entities_to_remove.remove(parent_node);
                    }
                    None => { }
                };
            }
        }    
        
        // Clear the list just in case
        if self.chunks_to_generate.len() == 0 { self.entities_to_remove.clear(); }

        // Now refresh our list
        self.parent_children_added_entity_chunks.retain(|coord, _| {
            // Remove the specified parents from the list
            !parents_removed.contains(coord)
        });
        println!("{}", self.entities_to_remove.len());
        return (new_chunks, entities_to_remove.iter().map(|x| *x).collect::<Vec<u16>>());
    }
}
