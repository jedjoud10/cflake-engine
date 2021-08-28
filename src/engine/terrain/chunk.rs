use super::{
    terrain::{Terrain, CHUNK_SIZE},
    voxel::{Voxel, VoxelGenerator},
};
use crate::engine::{
    core::ecs::component::{Component, ComponentID, ComponentInternal},
    rendering::model::{Model, ProceduralModelGenerator},
    terrain::tables::{DATA_OFFSET_TABLE, EDGE_TABLE, TRI_TABLE, VERTEX_TABLE},
};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

// A component that will be added to well... chunks
pub struct Chunk {
    pub position: veclib::Vector3<i32>,
    pub size: u32,
    pub data: Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            position: veclib::Vector3::<i32>::default_zero(),
            size: 0,
            data: Box::new([Voxel::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]),
        }
    }
}

// Main traits implemented
impl ComponentInternal for Chunk {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl ComponentID for Chunk {
    fn get_component_name() -> String {
        String::from("Chunk")
    }
}
impl Component for Chunk {}

impl Chunk {
    // Casually stole my old code lol
    // Get the position from an index
    fn unflatten(mut index: usize) -> (usize, usize, usize) {
        let z = index / (CHUNK_SIZE);
        index -= z * (CHUNK_SIZE);
        let y = index / (CHUNK_SIZE * CHUNK_SIZE);
        let x = index % (CHUNK_SIZE);
        return (x, y, z);
    }
    // Get the index from a position
    fn flatten(position: (usize, usize, usize)) -> usize {
        return position.0 + (position.1 * CHUNK_SIZE * CHUNK_SIZE) + (position.2 * CHUNK_SIZE);
    }
    // Generate the voxel data needed for mesh construction
    pub fn generate_data(&mut self, voxel_generator: &VoxelGenerator) -> (f32, f32) {
        let mut i = 0;
        let mut min: f32 = f32::MAX;
        let mut max: f32 = f32::MIN;
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    // Get the point in world coordinates
                    let size = self.size as f32 / (CHUNK_SIZE as f32 - 2.0);
                    let point: veclib::Vector3<f32> = veclib::Vector3::<f32>::new(x as f32, y as f32, z as f32) * size + veclib::Vector3::<f32>::from(self.position);
                    // Set the voxel data
                    self.data[i] = voxel_generator.get_voxel(point);
                    // Keep track of the min max values
                    min = min.min(self.data[i].density);
                    max = max.max(self.data[i].density);
                    i += 1;
                }
            }
        }
        return (min, max);
    }
}

// This is a procedural model generator
impl ProceduralModelGenerator for Chunk {
    // Generate a procedural marching cube model
    fn generate_model(&self) -> Model {
        let mut model: Model = Model::default();
        let mut duplicate_vertices: HashMap<(u32, u32, u32), u32> = HashMap::new();
        // Loop over every voxel
        for x in 0..CHUNK_SIZE - 2 {
            for y in 0..CHUNK_SIZE - 2 {
                for z in 0..CHUNK_SIZE - 2 {
                    let i = Self::flatten((x, y, z));
                    // Calculate the 8 bit number at that voxel position, so get all the 8 neighboring voxels
                    let mut case_index = 0u8;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[0]].density > 0.0) as u8) * 1;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[1]].density > 0.0) as u8) * 2;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[2]].density > 0.0) as u8) * 4;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[3]].density > 0.0) as u8) * 8;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[4]].density > 0.0) as u8) * 16;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[5]].density > 0.0) as u8) * 32;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[6]].density > 0.0) as u8) * 64;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[7]].density > 0.0) as u8) * 128;
                    // Get triangles
                    let edges: [i8; 16] = TRI_TABLE[case_index as usize];
                    for edge in edges {
                        // Make sure the triangle is valid
                        if edge != -1 {
                            // Get the vertex in local space
                            let vert1 = VERTEX_TABLE[EDGE_TABLE[(edge as usize) * 2]];
                            let vert2 = VERTEX_TABLE[EDGE_TABLE[(edge as usize) * 2 + 1]];

                            // In global space here
                            let vert1_usize = (vert1.x() as usize + x, vert1.y() as usize + y, vert1.z() as usize + z);
                            let vert2_usize = (vert2.x() as usize + x, vert2.y() as usize + y, vert2.z() as usize + z);
                            let index1 = Self::flatten(vert1_usize);
                            let index2 = Self::flatten(vert2_usize);
                            let density1 = self.data[index1].density;
                            let density2 = self.data[index2].density;
                            // Do inverse linear interpolation to find the factor value
                            let mut value: f32 = inverse_lerp(density1, density2, 0.0);
                            //value = 0.5;
                            // Create the vertex
                            let mut vertex = veclib::Vector3::<f32>::lerp(vert1, vert2, value);
                            // Offset the vertex
                            vertex += veclib::Vector3::<f32>::new(x as f32, y as f32, z as f32);
                            let normal: veclib::Vector3<f32> = {
                                let mut normal1 = veclib::Vector3::<f32>::default_zero();
                                let mut normal2 = veclib::Vector3::<f32>::default_zero();

                                // Create the normal
                                normal1.set_x(self.data[index1 + DATA_OFFSET_TABLE[3]].density - density1);
                                normal1.set_y(self.data[index1 + DATA_OFFSET_TABLE[4]].density - density1);
                                normal1.set_z(self.data[index1 + DATA_OFFSET_TABLE[1]].density - density1);
                                normal2.set_x(self.data[index2 + DATA_OFFSET_TABLE[3]].density - density2);
                                normal2.set_y(self.data[index2 + DATA_OFFSET_TABLE[4]].density - density2);
                                normal2.set_z(self.data[index2 + DATA_OFFSET_TABLE[1]].density - density2);
                                veclib::Vector3::<f32>::lerp(normal1, normal2, value)
                            };

                            let edge_tuple: (u32, u32, u32) = (
                                2 * x as u32 + vert1.x() as u32 + vert2.x() as u32,
                                2 * y as u32 + vert1.y() as u32 + vert2.y() as u32,
                                2 * z as u32 + vert1.z() as u32 + vert2.z() as u32,
                            );

                            // Check if this vertex was already added
                            if let Entry::Vacant(e) = duplicate_vertices.entry(edge_tuple) {
                                // Add this vertex
                                e.insert(model.vertices.len() as u32);
                                model.triangles.push(model.vertices.len() as u32);
                                model.vertices.push(vertex);
                                model.uvs.push(veclib::Vector2::<f32>::default_zero());
                                model.normals.push(normal.normalized());
                                model.tangents.push(veclib::Vector4::<f32>::default_zero());
                            } else {
                                // The vertex already exists
                                model.triangles.push(duplicate_vertices[&edge_tuple]);
                            }
                        }
                    }
                }
            }
        }
        // Inverse of lerp
        fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
            (x - a) / (b - a)
        }
        // Return the model
        model
    }
}
