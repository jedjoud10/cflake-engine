use std::collections::HashMap;

use super::terrain::{Terrain, CHUNK_SIZE};
use crate::engine::{
    core::ecs::component::{Component, ComponentID},
    rendering::model::{Model, ProceduralModelGenerator},
    terrain::tables::{EDGE_TABLE, TRI_TABLE, VERTEX_TABLE},
};

// A component that will be added to well... chunks
#[derive(Default)]
pub struct Chunk {
    pub position: glam::Vec3,
    pub data: [[[f32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub isoline: f32,
}

// Main traits implemented
impl Component for Chunk {
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

// Actual model generation
impl Chunk {
    // Generate the voxel data
    pub fn generate_data(&mut self, terrain_generator: &Terrain) {
        // Create the data using SIMD
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let data: f32 = y as f32 - 10.0;
                    self.data[x][y][z] = data;
                }
            }
        }

        // Save the isoline
        self.isoline = terrain_generator.isoline;
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
                    // Calculate the 8 bit number at that voxel position, so get all the 8 neighboring voxels
                    let mut case_index = 0u8;
                    case_index += ((self.data[x][y][z] > self.isoline) as u8) * 1;
                    case_index += ((self.data[x][y][z + 1] > self.isoline) as u8) * 2;
                    case_index += ((self.data[x + 1][y][z + 1] > self.isoline) as u8) * 4;
                    case_index += ((self.data[x + 1][y][z] > self.isoline) as u8) * 8;
                    case_index += ((self.data[x][y + 1][z] > self.isoline) as u8) * 16;
                    case_index += ((self.data[x][y + 1][z + 1] > self.isoline) as u8) * 32;
                    case_index += ((self.data[x + 1][y + 1][z + 1] > self.isoline) as u8) * 64;
                    case_index += ((self.data[x + 1][y + 1][z] > self.isoline) as u8) * 128;
                    // Get triangles
                    let edges: [i8; 16] = TRI_TABLE[case_index as usize];
                    for edge in edges {
                        // Make sure the triangle is valid
                        if edge != -1 {
                            // Get the vertex in local space
                            let vert1 = VERTEX_TABLE[EDGE_TABLE[(edge as usize) * 2]];
                            let vert2 = VERTEX_TABLE[EDGE_TABLE[(edge as usize) * 2 + 1]];

                            // In global space here
                            let vert1_usize = (
                                vert1.x as usize + x,
                                vert1.y as usize + y,
                                vert1.z as usize + z,
                            );
                            let vert2_usize = (
                                vert2.x as usize + x,
                                vert2.y as usize + y,
                                vert2.z as usize + z,
                            );
                            let density1 = self.data[vert1_usize.0][vert1_usize.1][vert1_usize.2];
                            let density2 = self.data[vert2_usize.0][vert2_usize.1][vert2_usize.2];
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
                                normal1.x = self.data[vert1_usize.0 + 1][vert1_usize.1]
                                    [vert1_usize.2]
                                    - density1;
                                normal1.y = self.data[vert1_usize.0][vert1_usize.1 + 1]
                                    [vert1_usize.2]
                                    - density1;
                                normal1.z = self.data[vert1_usize.0][vert1_usize.1]
                                    [vert1_usize.2 + 1]
                                    - density1;
                                normal2.x = self.data[vert2_usize.0 + 1][vert2_usize.1]
                                    [vert2_usize.2]
                                    - density2;
                                normal2.y = self.data[vert2_usize.0][vert2_usize.1 + 1]
                                    [vert2_usize.2]
                                    - density2;
                                normal2.z = self.data[vert2_usize.0][vert2_usize.1]
                                    [vert2_usize.2 + 1]
                                    - density2;
                                glam::Vec3::lerp(normal1, normal2, value)
                            };

                            let edge_tuple: (u32, u32, u32) = (
                                2 * x as u32 + vert1.x as u32 + vert2.x as u32,
                                2 * y as u32 + vert1.y as u32 + vert2.y as u32,
                                2 * z as u32 + vert1.z as u32 + vert2.z as u32,
                            );

                            // Check if this vertex was already added
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                duplicate_vertices.entry(edge_tuple)
                            {
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
