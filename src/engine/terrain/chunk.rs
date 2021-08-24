use super::{terrain::{Terrain, CHUNK_SIZE}, voxel::{Voxel, VoxelGenerator}};
use std::collections::hash_map::Entry;
use crate::engine::{core::ecs::component::{Component, ComponentID, ComponentInternal}, rendering::model::{Model, ProceduralModelGenerator}, terrain::tables::{DATA_OFFSET_TABLE, EDGE_TABLE, TRI_TABLE, VERTEX_TABLE}};
use std::collections::HashMap;

// A component that will be added to well... chunks
pub struct Chunk {
    pub position: glam::IVec3,
    pub size: u16,
    pub data: Box<[Voxel; (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize]>,
    pub isoline: f32,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            position: glam::IVec3::ZERO,
            size: 0,
            data: Box::new([Voxel::default(); (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize]),
            isoline: 0.0,
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
        let z = index / ((CHUNK_SIZE) * (CHUNK_SIZE));
        index -= z * (CHUNK_SIZE) * (CHUNK_SIZE);
        let y = index / (CHUNK_SIZE);
        let x = index % (CHUNK_SIZE);
        return (x, y, z);
    }
    // Get the index from a position
    fn flatten(position: (usize, usize, usize)) -> usize {
        return position.0 * CHUNK_SIZE * CHUNK_SIZE + (position.1 * CHUNK_SIZE) + position.2;
    }
    // Generate the voxel data needed for mesh construction
    pub fn generate_data(&mut self, voxel_generator: &VoxelGenerator) {
        for i in 0..(CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) {
            let local_point = Self::unflatten(i);
            // Get the point in world coordinates
            let point: glam::Vec3 = glam::vec3(local_point.0 as f32, local_point.1 as f32, local_point.2 as f32) + self.position.as_f32();
            // Set the voxel data
            self.data[0]= voxel_generator.get_voxel(glam::vec3(local_point.0 as f32, local_point.1 as f32, local_point.2 as f32));                
        }
    }
}

// This is a procedural model generator
impl ProceduralModelGenerator for Chunk {
    // Generate a procedural marching cube model
    fn generate_model(&self) -> Model {
        let mut model: Model = Model::default();
        let mut duplicate_vertices: HashMap<(u32, u32, u32), u32> = HashMap::new();
        let mut i: usize = 0;
        // Loop over every voxel
        for y in 0..CHUNK_SIZE - 2 {
            for z in 0..CHUNK_SIZE - 2 {
                for x in 0..CHUNK_SIZE - 2 {
                    // Calculate the 8 bit number at that voxel position, so get all the 8 neighboring voxels
                    let mut case_index = 0u8;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[0]].density > self.isoline) as u8) * 1;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[1]].density > self.isoline) as u8) * 2;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[2]].density > self.isoline) as u8) * 4;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[3]].density > self.isoline) as u8) * 8;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[4]].density > self.isoline) as u8) * 16;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[5]].density > self.isoline) as u8) * 32;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[6]].density > self.isoline) as u8) * 64;
                    case_index += ((self.data[i + DATA_OFFSET_TABLE[7]].density > self.isoline) as u8) * 128;
                    // Get triangles
                    let edges: [i8; 16] = TRI_TABLE[case_index as usize];
                    for edge in edges {
                        // Make sure the triangle is valid
                        if edge != -1 {
                            // Get the vertex in local space
                            let vert1 = VERTEX_TABLE[EDGE_TABLE[(edge as usize) * 2]];
                            let vert2 = VERTEX_TABLE[EDGE_TABLE[(edge as usize) * 2 + 1]];

                            // In global space here
                            let vert1_usize = (vert1.x as usize + x, vert1.y as usize + y, vert1.z as usize + z);
                            let vert2_usize = (vert2.x as usize + x, vert2.y as usize + y, vert2.z as usize + z);
                            let index1 = Self::flatten(vert1_usize);
                            let index2 = Self::flatten(vert2_usize);
                            let density1 = self.data[index1].density;
                            let density2 = self.data[index2].density;
                            // Do inverse linear interpolation to find the factor value
                            let value: f32 = inverse_lerp(density1, density2, self.isoline);

                            // Create the vertex
                            let mut vertex = glam::Vec3::lerp(vert1, vert2, value);
                            // Offset the vertex
                            vertex += glam::vec3(x as f32, y as f32, z as f32);
                            let normal: glam::Vec3 = {
                                let mut normal1 = glam::Vec3::ZERO;
                                let mut normal2 = glam::Vec3::ZERO;

                                // Create the normal
                                normal1.x = self.data[index1 + DATA_OFFSET_TABLE[3]].density - density1;
                                normal1.y = self.data[index1 + DATA_OFFSET_TABLE[4]].density - density1;
                                normal1.z = self.data[index1 + DATA_OFFSET_TABLE[1]].density - density1;
                                normal2.x = self.data[index2 + DATA_OFFSET_TABLE[3]].density - density2;
                                normal2.y = self.data[index2 + DATA_OFFSET_TABLE[4]].density - density2;
                                normal2.z = self.data[index2 + DATA_OFFSET_TABLE[1]].density - density2;
                                glam::Vec3::lerp(normal1, normal2, value)
                            };

                            let edge_tuple: (u32, u32, u32) = (
                                2 * x as u32 + vert1.x as u32 + vert2.x as u32,
                                2 * y as u32 + vert1.y as u32 + vert2.y as u32,
                                2 * z as u32 + vert1.z as u32 + vert2.z as u32,
                            );

                            // Check if this vertex was already added
                            if let Entry::Vacant(e) = duplicate_vertices.entry(edge_tuple) {
                                // Add this vertex
                                e.insert(model.vertices.len() as u32);
                                model.triangles.push(model.vertices.len() as u32);
                                model.vertices.push(vertex);
                                model.uvs.push(glam::Vec2::ZERO);
                                model.normals.push(normal.normalize());
                                model.tangents.push(glam::Vec4::ZERO);
                            } else {
                                // The vertex already exists
                                model.triangles.push(duplicate_vertices[&edge_tuple]);
                            }
                        }
                    }
                    i  += 1;
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
