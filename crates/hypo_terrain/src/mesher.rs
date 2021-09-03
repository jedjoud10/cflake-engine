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

                // Local edges for the X axis
                let mut local_edges_x: [(u32, u32, u32); 4] = [(0, 0, 0); 4];
                let mut local_edges_hit_x: bool = false;
                // Local edges for the X axis
                let mut local_edges_y: [(u32, u32, u32); 4] = [(0, 0, 0); 4];
                let mut local_edges_hit_y: bool = false;
                // Local edges for the X axis
                let mut local_edges_z: [(u32, u32, u32); 4] = [(0, 0, 0); 4];
                let mut local_edges_hit_z: bool = false;
                

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

                        // This edge is at the X base
                        if vert1_usize.0 == 0 && vert2_usize.0 == 0 {
                            local_edges_x[MC_EDGES_TO_LOCAL_VERTS_X[edge as usize] as usize] = edge_tuple;
                            local_edges_hit_x = true;
                        }
                        // This edge is at the Y base
                        if vert1_usize.1 == 0 && vert2_usize.1 == 0 {
                            local_edges_y[MC_EDGES_TO_LOCAL_VERTS_Y[edge as usize] as usize] = edge_tuple;
                            local_edges_hit_y = true;
                        }                        
                        // This edge is at the Z base
                        if vert1_usize.2 == 0 && vert2_usize.2 == 0 {
                            local_edges_z[MC_EDGES_TO_LOCAL_VERTS_Z[edge as usize] as usize] = edge_tuple;
                            local_edges_hit_z = true;
                        }                        
                    }
                }
            
                // If this is the base skirt X
                //if local_edges_hit_x { solve_marching_squares(y, z, i, &data, &local_edges_x, &mut shared_vertices, veclib::Vec3Axis::X, DENSITY_OFFSET_X); }
                //if local_edges_hit_y { solve_marching_squares(x, z, i, &data, &local_edges_y, &mut shared_vertices, veclib::Vec3Axis::Y, DENSITY_OFFSET_Y); }
                if local_edges_hit_z { solve_marching_squares(y, x, i, &data, &local_edges_z, &mut shared_vertices, veclib::Vec3Axis::Z, DENSITY_OFFSET_Z); }
            }
        }    
    }    

    // Turn the shared vertices into triangle indices
    for shared_vertex in shared_vertices.iter() {
        match shared_vertex {
            SkirtVertex::Vertex(vertex) => {
                // This vertex isn't a shared vertex
                println!("Vert: {:?}", vertex);
                skirts_model.triangles.push(skirts_model.vertices.len() as u32 + model.vertices.len() as u32);
                skirts_model.vertices.push(vertex.clone());
            },
            SkirtVertex::SharedVertex(coord_tuple) => {    
                let tri = duplicate_vertices[coord_tuple];
                println!("Shared: {:?}", model.vertices[duplicate_vertices[coord_tuple] as usize]);
                // This vertex is a vertex that already exists in the main model
                skirts_model.triangles.push(tri);
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


// Solve a single marching squares case using a passed function for 
pub fn solve_marching_squares(a: usize, b: usize, i: usize, data: &Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>, local_edges: &[(u32, u32, u32); 4], shared_vertices: &mut Vec<SkirtVertex>, axis: veclib::Vec3Axis, density_offset: [usize; 4]) {
    println!("Second");
    let mut case = 0_u8;
    // For axis X:
    //  3---2
    //  |   |
    //  |   |
    //  0---1
    case += (!(data[i + density_offset[0]].density > 0.0) as u8) * 1;
    case += (!(data[i + density_offset[1]].density > 0.0) as u8) * 2;
    case += (!(data[i + density_offset[2]].density > 0.0) as u8) * 4;
    case += (!(data[i + density_offset[3]].density > 0.0) as u8) * 8;
    println!("{}", case);
    // Skip the full and empty cases
    if case == 0 || case == 15 {
        return;
    }
    let offset = veclib::Vector2::<f32>::new(a as f32, b as f32);
    // The vertices to connect
    let tris = super::SQUARES_TRI_TABLE[case as usize];    
    for tri_group in 0..3 {
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
                            let index = (tri - 1) / 2;
                            let edge_tuple = local_edges[index as usize];
                            shared_vertices.push(SkirtVertex::SharedVertex(edge_tuple));
                            println!("Run");
                            //tri_global_switched[tri_i] = skirt_vert_indices[&edge_tuple];
                        }
                        _ => {}
                    }                            
                } else {
                    // Check if this vertex was already added
                    //tri_global_switched[tri_i] = model.triangles.len() as u32 + skirts_model.vertices.len() as u32;
                    // This is a vertex that is not present in the main mesh    
                    let new_vertex: veclib::Vector3<f32> = match axis {
                        veclib::Vec3Axis::X => transform_x_local(0, &vertex, &offset),
                        veclib::Vec3Axis::Y => transform_y_local(0, &vertex, &offset),
                        veclib::Vec3Axis::Z => transform_z_local(0, &vertex, &offset),
                    };
                    println!("Before {:?} After {:?}", vertex, new_vertex);
                    shared_vertices.push(SkirtVertex::Vertex(new_vertex));
                }           
            }
        }  
    }
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the X axis
fn transform_x_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(slice as f32, vertex.y() + offset.x(), vertex.x() + offset.y())
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the Y axis
fn transform_y_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vertex.x() + offset.x(), slice as f32, vertex.y() + offset.y())
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the Z axis
fn transform_z_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vertex.y() + offset.y(), vertex.x() + offset.x(), slice as f32)
}