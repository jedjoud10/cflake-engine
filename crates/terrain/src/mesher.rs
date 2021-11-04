use crate::MAIN_CHUNK_SIZE;
use crate::TCase;
use crate::TModel;
use crate::ISOLINE;

use super::tables::*;
use super::Voxel;
use rendering::Model;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::Instant;

// Inverse of lerp
fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
    (x - a) / (b - a)
}

// Generate the Marching Cubes model
pub fn generate_model(voxels: &Box<[Voxel]>, size: usize, interpolation: bool, skirts: bool) -> TModel {
    let mut duplicate_vertices: HashMap<(u32, u32, u32, u8), u32> = HashMap::new();
    let mut sub_model_hashmap: HashMap<u8, (Model, Vec<SkirtVertex>)> = HashMap::new();
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
                sub_model_hashmap.entry(lv.shader_id).or_insert((Model::default(), Vec::new()));
                let (model, shared_vertices) = sub_model_hashmap.get_mut(&lv.shader_id).unwrap();
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
                        let value: f32 = if interpolation { inverse_lerp(voxel1.density, voxel2.density, ISOLINE as f32) } else { 0.5 };
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
                            } else { n }
                        };

                        // Get the color
                        let color: veclib::Vector3<f32> = { veclib::Vector3::<f32>::lerp(voxel1.color.into(), voxel2.color.into(), value) } / 255.0;

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
                            model.uvs.push(veclib::Vector2::<f32>::ZERO);
                            let c: veclib::Vector3<f32> = lv.color.into();
                            model.colors.push(color);
                            model.normals.push(normal.normalized());
                            model.tangents.push(veclib::Vector4::<f32>::ZERO);
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
    /*
    // Create the skirts in a completely separate loop
    for x in 0..MAIN_CHUNK_SIZE {
        for y in 0..MAIN_CHUNK_SIZE {
            
        }
    }
    // The skirts' models
    let mut skirt_models: HashMap<u8, Model> = HashMap::new();
    for (shader_id, (model, shared_vertices)) in sub_model_hashmap.iter() {
        // Make sure the skirts model exists
        skirt_models.entry(*shader_id).or_default();
        let skirt_model = skirt_models.get_mut(shader_id).unwrap();
        // Turn the shared vertices into triangle indices
        for shared_vertex in shared_vertices {
            match shared_vertex {
                SkirtVertex::Vertex(vertex, normal, color) => {
                    // This vertex isn't a shared vertex
                    skirt_model.triangles.push(skirt_model.vertices.len() as u32);
                    skirt_model.vertices.push(vertex.clone());
                    skirt_model.normals.push(normal.clone());
                    skirt_model.colors.push(color.clone());
                }
                SkirtVertex::SharedVertex(coord_tuple) => {
                    let tri = *duplicate_vertices.get(&(coord_tuple.0, coord_tuple.1, coord_tuple.2, *shader_id)).unwrap();
                    // Get the vertex, and duplicate it, since the skirts are in their own sub model
                    let vert_data = (model.vertices[tri as usize], model.normals[tri as usize], model.colors[tri as usize]);
                    skirt_model.triangles.push(skirt_model.vertices.len() as u32);
                    skirt_model.vertices.push(vert_data.0);
                    skirt_model.normals.push(vert_data.1);
                    skirt_model.colors.push(vert_data.2);
                }
            }
        }
    }
    */
    let new_model_hashmap = sub_model_hashmap
        .into_iter()
        .map(|(shader_id, (model, _))| (shader_id, model))
        .collect::<HashMap<u8, Model>>();
    // Return the model
    return TModel {
        shader_model_hashmap: new_model_hashmap,
        skirt_models: HashMap::new(),
        intersection_cases: Some(intersection_cases),
    };
}

// The type of skirt vertex, normal or shared
pub enum SkirtVertex {
    Vertex(veclib::Vector3<f32>, veclib::Vector3<f32>, veclib::Vector3<f32>),
}

// Solve a single marching squares case using a passed function for
pub fn solve_marching_squares(
    data: &Box<[Voxel]>,
    axis: veclib::Vec3Axis,
    slice: usize,
    density_offset: [usize; 4],
    flip: bool,
) -> Option<Vec<SkirtVertex>> {    
    // Gotta reprogram this
    return None;
}