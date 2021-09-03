use super::CHUNK_SIZE;
use super::tables::*;
use super::Voxel;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use hypo_rendering::Model;

// Inverse of lerp
fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
    (x - a) / (b - a)
}

// Generate the Marching Cubes model
pub fn generate_model(data: &Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>) -> Model {
    let mut model: Model = Model::default();
    let mut skirts_model: Model = Model::default();
    let mut duplicate_vertices: HashMap<(u32, u32, u32), u32> = HashMap::new();
    let mut shared_vertices: Vec<SkirtVertex> = Vec::new();
    // Loop over every voxel
    for x in 0..CHUNK_SIZE - 2 {
        for y in 0..CHUNK_SIZE - 2 {
            for z in 0..CHUNK_SIZE - 2 {
                let i = super::flatten((x, y, z));
                // Calculate the 8 bit number at that voxel position, so get all the 8 neighboring voxels
                let mut case_index = 0u8;                
                case_index += ((data[i + DATA_OFFSET_TABLE[0]].density > 0.0) as u8) * 1;
                case_index += ((data[i + DATA_OFFSET_TABLE[1]].density > 0.0) as u8) * 2;
                case_index += ((data[i + DATA_OFFSET_TABLE[2]].density > 0.0) as u8) * 4;
                case_index += ((data[i + DATA_OFFSET_TABLE[3]].density > 0.0) as u8) * 8;
                case_index += ((data[i + DATA_OFFSET_TABLE[4]].density > 0.0) as u8) * 16;
                case_index += ((data[i + DATA_OFFSET_TABLE[5]].density > 0.0) as u8) * 32;
                case_index += ((data[i + DATA_OFFSET_TABLE[6]].density > 0.0) as u8) * 64;
                case_index += ((data[i + DATA_OFFSET_TABLE[7]].density > 0.0) as u8) * 128;

                // Skip the completely empty and completely filled cases
                if case_index == 0 || case_index == 255 {
                    //continue;
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
                        let vert1_usize = (vert1.x() as usize + x, vert1.y() as usize + y, vert1.z() as usize + z);
                        let vert2_usize = (vert2.x() as usize + x, vert2.y() as usize + y, vert2.z() as usize + z);
                        let index1 = super::flatten(vert1_usize);
                        let index2 = super::flatten(vert2_usize);
                        let density1 = data[index1].density;
                        let density2 = data[index2].density;
                        // Do inverse linear interpolation to find the factor value
                        let value: f32 = inverse_lerp(density1, density2, 0.0);
                        // Create the vertex
                        let mut vertex = veclib::Vector3::<f32>::lerp(vert1, vert2, value);
                        // Offset the vertex
                        vertex += veclib::Vector3::<f32>::new(x as f32, y as f32, z as f32);
                        let normal: veclib::Vector3<f32> = {
                            let mut normal1 = veclib::Vector3::<f32>::default_zero();
                            let mut normal2 = veclib::Vector3::<f32>::default_zero();

                            // Create the normal
                            normal1.set_x(data[index1 + DATA_OFFSET_TABLE[3]].density - density1);
                            normal1.set_y(data[index1 + DATA_OFFSET_TABLE[4]].density - density1);
                            normal1.set_z(data[index1 + DATA_OFFSET_TABLE[1]].density - density1);
                            normal2.set_x(data[index2 + DATA_OFFSET_TABLE[3]].density - density2);
                            normal2.set_y(data[index2 + DATA_OFFSET_TABLE[4]].density - density2);
                            normal2.set_z(data[index2 + DATA_OFFSET_TABLE[1]].density - density2);
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

                        if vert1_usize.0 == 0 && vert2_usize.0 == 0 {
                            println!("First");
                            println!("A: {:?}", edge_tuple);
                            println!("B: {} {}", 2 * y, 2 * x);
                        }
                    }
                }
            
                // If this is the base skirt X
                if x == 0 {
                    println!("Second");
                    let mut case = 0_u8;
                    //  3---2
                    //  |   |
                    //  |   |
                    //  0---1
                    case += ((data[i + DATA_OFFSET_TABLE[0]].density < 0.0) as u8) * 1;
                    case += ((data[i + DATA_OFFSET_TABLE[1]].density < 0.0) as u8) * 2;
                    case += ((data[i + DATA_OFFSET_TABLE[5]].density < 0.0) as u8) * 4;
                    case += ((data[i + DATA_OFFSET_TABLE[4]].density < 0.0) as u8) * 8;
                    // Skip the full and empty cases
                    if case == 0 || case == 15 {
                        //continue;
                    }
                    let offset = veclib::Vector2::<f32>::new(y as f32, z as f32);
                    // The vertices to connect
                    let tris = super::SQUARES_TRI_TABLE[case as usize];    
                    for tri_group in 0..3 {
                        let mut hit: bool = false;
                        for tri_i in 0..3 {
                            let tri = tris[tri_i+tri_group*3];
                            // Check if the value is negative first
                            if tri != -1 {
                                // The bertex
                                let vertex = SQUARES_VERTEX_TABLE[tri as usize];
                                // Interpolation            
                                if vertex == -veclib::Vector2::default_one() {    
                                    match tri {
                                        // TODO: Turn this into a more generalized algorithm
                                        1 | 3 | 5 | 7 => {             
                                            let last = SQUARES_VERTEX_TABLE[tri as usize - 1];
                                            let next = SQUARES_VERTEX_TABLE[((tri - 1) % 8) as usize];      
                                            let edge_tuple: (u32, u32, u32) = (
                                                0,
                                                2 * y as u32 + last.x() as u32 + next.x() as u32,
                                                2 * z as u32 + last.y() as u32 + next.y() as u32,
                                            );
                                            println!("{} {}", 2 * y, 2 * z);
                                            shared_vertices.push(SkirtVertex::SharedVertex(edge_tuple));
                                            //tri_global_switched[tri_i] = skirt_vert_indices[&edge_tuple];
                                        }
                                        _ => {}
                                    }                            
                                } else {
                                    // Check if this vertex was already added
                                    //tri_global_switched[tri_i] = model.triangles.len() as u32 + skirts_model.vertices.len() as u32;
                                    // This is a vertex that is not present in the main mesh                  
                                    let vertex = veclib::Vector3::<f32>::new(0 as f32, vertex.y() + offset.x(), vertex.x() + offset.y());
                                    shared_vertices.push(SkirtVertex::Vertex(vertex));
                                }               
                                hit = true;
                            }
                        }        
                        if hit {
                            /*
                            // Flip the triangle 
                            if flip {
                                // Swap the first and last indices
                                //tri_global_switched.swap(0, 2);
                            }
                            // Add it
                            triangles.extend(tri_global_switched);
                            */
                        }
                    }
                }
            }
        }    
    }    

    // Turn the shared vertices into triangle indices
    for shared_vertex in shared_vertices.iter() {
        match shared_vertex {
            SkirtVertex::Vertex(vertex) => {
                // This vertex isn't a shared vertex
                skirts_model.vertices.push(vertex.clone());
                skirts_model.triangles.push(skirts_model.triangles.len() as u32);
            },
            SkirtVertex::SharedVertex(coord_tuple) => {                
                println!("{:?}", coord_tuple);
                // This vertex is a vertex that already exists in the main model
                skirts_model.triangles.push(duplicate_vertices[coord_tuple]);
            },
        }
    } 

    // Create the X skirt
    /*
    let skirt_end_x = generate_skirt(data, veclib::Vector3::new(1.0, 0.0, 0.0), transform_x_local, get_local_data_x, CHUNK_SIZE - 2, true);
    let skirt_x = Model::combine(&skirt_base_x, &skirt_end_x);
    // Create the Y skirt
    let skirt_base_y = generate_skirt(data, veclib::Vector3::new(0.0, -1.0, 0.0), transform_y_local, get_local_data_y, 0, true);
    let skirt_end_y = generate_skirt(data, veclib::Vector3::new(0.0, 1.0, 0.0), transform_y_local, get_local_data_y, CHUNK_SIZE - 2, false);
    let skirt_y = Model::combine(&skirt_base_y, &skirt_end_y);
    // Create the Y skirt
    let skirt_base_z = generate_skirt(data, veclib::Vector3::new(0.0, 0.0, -1.0), transform_z_local, get_local_data_z, 0, false);
    let skirt_end_z = generate_skirt(data, veclib::Vector3::new(0.0, 0.0, 1.0), transform_z_local, get_local_data_z, CHUNK_SIZE - 2, true);
    let skirt_z = Model::combine(&skirt_base_z, &skirt_end_z);
    */
    model = model.combine_smart(&skirts_model);
    //model = model.combine_smart(&skirt_y);
    //model = model.combine_smart(&skirt_z);
    // Return the model
    model
}

// The type of skirt vertex, normal or shared
pub enum SkirtVertex {
    Vertex(veclib::Vector3<f32>),
    SharedVertex((u32, u32, u32))
}


// TODO: Turn this better
// Get the local data for the X axis using a origin coordinate
pub fn get_local_data_x(data: &Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>, origin: (usize, usize), slice: usize) -> [f32; 4] {
    let local_data: [f32; 4] = [
        data[super::flatten((slice, origin.0, origin.1))].density,
        data[super::flatten((slice, origin.0, origin.1 + 1))].density,
        data[super::flatten((slice, origin.0 + 1, origin.1 + 1))].density,
        data[super::flatten((slice, origin.0 + 1, origin.1))].density       
    ];
    return local_data;
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the X axis
fn transform_x_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(slice as f32, vertex.y() + offset.x(), vertex.x() + offset.y())
}

// Get the local data for the Y axis using a origin coordinate
pub fn get_local_data_y(data: &Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>, origin: (usize, usize), slice: usize) -> [f32; 4] {
    let local_data: [f32; 4] = [
        data[super::flatten((origin.0, slice, origin.1))].density,
        data[super::flatten((origin.0, slice, origin.1 + 1))].density,
        data[super::flatten((origin.0 + 1, slice, origin.1 + 1))].density,
        data[super::flatten((origin.0 + 1, slice, origin.1))].density       
    ];
    return local_data;
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the Y axis
fn transform_y_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vertex.y() + offset.x(), slice as f32, vertex.x() + offset.y())
}

// Get the local data for the Z axis using a origin coordinate
pub fn get_local_data_z(data: &Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>, origin: (usize, usize), slice: usize) -> [f32; 4] {
    let local_data: [f32; 4] = [
        data[super::flatten((origin.0, origin.1, slice))].density,
        data[super::flatten((origin.0, origin.1 + 1, slice))].density,
        data[super::flatten((origin.0 + 1, origin.1 + 1, slice))].density,
        data[super::flatten((origin.0 + 1, origin.1, slice))].density       
    ];
    return local_data;
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the Z axis
fn transform_z_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vertex.y() + offset.x(), vertex.x() + offset.y(), slice as f32)
}