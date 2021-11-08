use crate::TCase;
use crate::TModel;
use crate::ISOLINE;
use crate::MAIN_CHUNK_SIZE;

use super::tables::*;
use super::Voxel;
use rendering::Model;
use std::collections::hash_map::Entry;
use std::collections::HashMap;


// Inverse of lerp
fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
    (x - a) / (b - a)
}

// Generate the Marching Cubes model
pub fn generate_model(voxels: &Box<[Voxel]>, _size: usize, interpolation: bool, _skirts: bool) -> TModel {
    let mut duplicate_vertices: HashMap<(u32, u32, u32, u8), u32> = HashMap::new();
    let mut sub_model_hashmap: HashMap<u8, Model> = HashMap::new();
    let mut intersection_cases: Vec<TCase> = Vec::new();
    // Loop over every voxel
    for x in 0..MAIN_CHUNK_SIZE {
        for y in 0..MAIN_CHUNK_SIZE {
            for z in 0..MAIN_CHUNK_SIZE {
                let i = super::flatten((x, y, z));
                // Calculate the 8 bit number at that voxel position, so get all the 8 neighboring voxels
                let mut case_index = 0u8;
                // Leading Voxel
                let lv = voxels[i + DATA_OFFSET_TABLE[0]];

                // Make sure we have the default submodel/material for this material ID
                sub_model_hashmap.entry(lv.shader_id).or_insert(Model::default());
                let model = sub_model_hashmap.get_mut(&lv.shader_id).unwrap();
                case_index |= ((voxels[i + DATA_OFFSET_TABLE[0]].density >= ISOLINE) as u8) * 1;
                case_index |= ((voxels[i + DATA_OFFSET_TABLE[1]].density >= ISOLINE) as u8) * 2;
                case_index |= ((voxels[i + DATA_OFFSET_TABLE[2]].density >= ISOLINE) as u8) * 4;
                case_index |= ((voxels[i + DATA_OFFSET_TABLE[3]].density >= ISOLINE) as u8) * 8;
                case_index |= ((voxels[i + DATA_OFFSET_TABLE[4]].density >= ISOLINE) as u8) * 16;
                case_index |= ((voxels[i + DATA_OFFSET_TABLE[5]].density >= ISOLINE) as u8) * 32;
                case_index |= ((voxels[i + DATA_OFFSET_TABLE[6]].density >= ISOLINE) as u8) * 64;
                case_index |= ((voxels[i + DATA_OFFSET_TABLE[7]].density >= ISOLINE) as u8) * 128;

                // Skip the completely empty and completely filled cases
                if case_index == 0 || case_index == 255 {
                    continue;
                }
                // Get triangles
                let edges: [i8; 16] = TRI_TABLE[case_index as usize];

                // The vertex indices that are gonna be used for the skirts
                for edge in edges {
                    // Make sure the triangle is valid
                    if edge != -1 {
                        // Get the vertex in local space
                        let vert1 = VERTEX_TABLE[EDGE_TABLE[(edge as usize) * 2]];
                        let vert2 = VERTEX_TABLE[EDGE_TABLE[(edge as usize) * 2 + 1]];

                        // In global space here
                        let vert1_usize = (vert1.x as usize + x, vert1.y as usize + y, vert1.z as usize + z);
                        let vert2_usize = (vert2.x as usize + x, vert2.y as usize + y, vert2.z as usize + z);
                        let index1 = super::flatten(vert1_usize);
                        let index2 = super::flatten(vert2_usize);
                        let voxel1 = voxels[index1];
                        let voxel2 = voxels[index2];
                        // Do inverse linear interpolation to find the factor value
                        let value: f32 = if interpolation {
                            inverse_lerp(voxel1.density, voxel2.density, ISOLINE as f32)
                        } else {
                            0.5
                        };
                        // Create the vertex
                        let mut vertex = veclib::Vector3::<f32>::lerp(vert1, vert2, value);
                        // Offset the vertex
                        vertex += veclib::Vector3::<f32>::new(x as f32, y as f32, z as f32);
                        // Get the normal
                        let normal: veclib::Vector3<f32> = {
                            // Do the lerping
                            let n = veclib::Vector3::<f32>::lerp(voxel1.normal, voxel2.normal, value);
                            if n == veclib::Vector3::ZERO {
                                veclib::Vector3::<f32>::lerp(voxel1.normal, voxel2.normal, 0.5)
                            } else {
                                n
                            }
                        };

                        // The edge tuple used to identify this vertex
                        let edge_tuple: (u32, u32, u32, u8) = (
                            2 * x as u32 + vert1.x as u32 + vert2.x as u32,
                            2 * y as u32 + vert1.y as u32 + vert2.y as u32,
                            2 * z as u32 + vert1.z as u32 + vert2.z as u32,
                            lv.shader_id,
                        );

                        // Check if this vertex was already added
                        if let Entry::Vacant(e) = duplicate_vertices.entry(edge_tuple) {
                            // Add this vertex
                            e.insert(model.vertices.len() as u32);
                            model.triangles.push(model.vertices.len() as u32);
                            model.vertices.push(vertex);
                            model.normals.push(normal.normalized());
                            model.uvs.push(veclib::Vector2::ZERO);
                            model.tangents.push(veclib::Vector4::ZERO);
                            model.colors.push(veclib::Vector3::ONE);
                        } else {
                            // The vertex already exists
                            model.triangles.push(duplicate_vertices[&edge_tuple]);
                        }
                    }
                }
                // Push this intersecting case
                intersection_cases.push(TCase {
                    cube_position: veclib::Vector3::<f32>::new(x as f32, y as f32, z as f32),
                    leading_voxel: lv,
                });
            }
        }
    }
    // Create the base-X skirt
    /*
    for x in 0..MAIN_CHUNK_SIZE {
        for y in 0..MAIN_CHUNK_SIZE {

        }
    }
    */
    
    // Return the model
    TModel {
        shader_model_hashmap: sub_model_hashmap,
        skirt_models: HashMap::new(),
        intersection_cases: Some(intersection_cases),
    }
}

// The type of skirt vertex, normal or shared
pub enum SkirtVertex {
    Vertex(veclib::Vector3<f32>, veclib::Vector3<f32>, veclib::Vector3<f32>),
}

// Solve a single marching squares case using a passed function for
pub fn solve_marching_squares(case: u8, _local_skirt_voxels: &[Voxel], _flip: bool) -> Option<Vec<SkirtVertex>> {
    // Create the triangles from the local skirts
    match case {
        1 => {}
        0 | 15 => { /* Empty cases */ }
        _ => { /* Case number is unsuported */ }
    }
    None
}
