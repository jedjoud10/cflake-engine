use std::collections::{HashMap, HashSet};

use debug::DefaultDebugRendererType;
use defaults::components;
use ecs::{ComponentManager, Entity};
use math::octree::OctreeNode;
use rendering::{Model, Shader};
use system_event_data::SystemEventData;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{CHUNK_SIZE, ChunkData, Terrain, VoxelGenerator, chunk_data::ChunkCoords, mesher};

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
    pub parent_children_added_entity_chunks: HashMap<veclib::Vector3<i64>, Vec<(ChunkCoords, Option<Model>)>>,
    pub parent_child_count: HashMap<veclib::Vector3<i64>, u8>,
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
    pub fn update(&mut self, voxel_generator: &VoxelGenerator, data: &mut SystemEventData, parent_child_count: HashMap<veclib::Vector3<i64>, u8>, nodes: HashMap<veclib::Vector3<i64>, OctreeNode>) -> (Vec<(ChunkCoords, Model)>, Vec<u16>) {
        // Check if we are currently generating the chunks
        self.parent_child_count = parent_child_count;
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
            let t = DefaultDebugRendererType::CUBE(veclib::Vector3::from(chunk_to_generate.center), veclib::Vector3::new(chunk_to_generate.size as f32, chunk_to_generate.size as f32, chunk_to_generate.size as f32));
            //data.debug.debug_default(t, veclib::Vector3::ONE, false);
        }
        // Generate the data for some chunks, then create their model
        let mut new_chunks: Vec<(ChunkCoords, Model)> = Vec::new();
        let coord = self.chunks_to_generate[0..(1.min(self.chunks_to_generate.len()))].get(0);

        let generated_chunk = match coord {
            Some(chunk_coords) => {
                // Get the chunk coords 
                let chunk_coords = chunk_coords.clone();
                let mut voxels: Box<[super::Voxel]> = Box::new([super::Voxel::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]);

                // Make sure the key exists
                self.parent_children_added_entity_chunks.entry(chunk_coords.parent_center).or_insert(Vec::new());
                
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

                        // If we don't have a surface, no need to create a model for this chunk
                        match has_surface {
                            Some(_) => {
                                // We have a surface, create the model
                                let coords = chunk_coords.clone();
                                let model = mesher::generate_model(&voxels, chunk_coords.size as usize, true, true);
                                // Save the chunk's data, though don't save the mode
                                let chunk_data = ChunkData { coords: coords, voxels: voxels };    
                                
                                self.parent_children_added_entity_chunks.entry(chunk_coords.parent_center).and_modify(|x| {
                                    x.push((chunk_coords.clone(), Some(model.clone())));
                                });
                                
                                Some((chunk_data, model))
                            }
                            None => {
                                // We don't have a surface, no need to create the model, but rerun the update loop to find a model that doe have a surface
                                self.parent_children_added_entity_chunks.entry(chunk_coords.parent_center).and_modify(|x| {
                                    x.push((chunk_coords.clone(), None));
                                });
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
        let mut valid_parents: HashSet<veclib::Vector3<i64>> = HashSet::new();
        // Check all the parents and check if their chunk child reached their limit
        for (parent_node, children) in self.parent_children_added_entity_chunks.iter() {
            // Check if it is equal the limit
            let limit = self.parent_child_count.get(parent_node).unwrap();
            if children.len() as u8 == *limit {
                // We have the correct amount of children, we can swap the children with their parent
                for child in children {
                    match &child.1 {
                        Some(model) => {
                            let coords = &child.0;
                            // Valid model
                            // Check if this node is below the max_depth, because if it is, we need to remove the nodes that are in it's area first
                            /*
                            if coords.depth < crate::terrain::OCTREE_DEPTH {
                                // This node has nodes that intersect it, probably
                                // Check each node that this node might intersect with
                                let aabb = math::bounds::AABB { min: coords.position.into(), max: veclib::Vector3::<f32>::from(coords.position) + coords.size as f32 };
                                for node in nodes.iter() {
                                    // Check intersection
                                    if math::Intersection::point_aabb(&veclib::Vector3::<f32>::from(node.1.get_center()), &aabb) {
                                        if self.entities_to_remove.contains_key(node.0) {
                                            let entity_id = self.entities_to_remove.get(node.0).unwrap();
                                            entities_to_remove.insert(*entity_id);
                                            self.entities_to_remove.remove(node.0);
                                        }
                                    }
                                }
                            }
                            */
                            // TODO: Gotta make this not clone the values
                            new_chunks.push((coords.clone(), model.clone()));
                            self.chunks.insert(coords.center);
                        }
                        None => { /* No need */ }
                    }
                }
                // Check if we should remove this parent node
                valid_parents.insert(*parent_node);
                let entity_id = self.entities_to_remove.get(parent_node);
                match entity_id {
                    Some(parent_id) => {
                        entities_to_remove.insert(*parent_id);
                        self.entities_to_remove.remove(parent_node);
                    }
                    None => { }
                };
            } else {
                //println!("{} {} {:?}", children.len(), limit, parent_node);
                for child in children {
                    let coords = &child.0;
                    data.debug.debug_default(DefaultDebugRendererType::CUBE(coords.center.clone().into(), veclib::Vector3::ONE * coords.size as f32 * 0.8), veclib::Vector3::X, false);
                }
            }
        }    

        // Remove the parents that got their children added to the world
        self.parent_children_added_entity_chunks.retain(|x, _| !valid_parents.contains(x));
        
        // Clear the list just in case
        if self.chunks_to_generate.len() == 0 { 
            let a = self.entities_to_remove.values().map(|x| *x).collect::<Vec<u16>>();
            entities_to_remove.extend(a);
            self.entities_to_remove.clear();
        }

        //println!("{}", self.parent_children_added_entity_chunks.len());
        return (new_chunks, entities_to_remove.iter().map(|x| *x).collect::<Vec<u16>>());
    }
}
