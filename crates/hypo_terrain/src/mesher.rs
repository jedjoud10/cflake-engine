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
    let mut duplicate_vertices: HashMap<(u32, u32, u32), u32> = HashMap::new();
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
                    continue;
                }
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

                        if vert1_usize.0 == 0 && vert2_usize.0 == 0 && (vert2_usize.1 > vert1_usize.1) {
                            println!("{:?}", vert1_usize);
                            println!("{:?}", vert2_usize);
                            println!("Start tuple: {:?} {:?}", edge_tuple, (y, z));
                            let edge_tuple: (u32, u32, u32) = (
                                vert1_usize.0 as u32 + vert2_usize.0 as u32,
                                vert1_usize.1 as u32 + vert2_usize.1 as u32,
                                vert1_usize.2 as u32 + vert2_usize.2 as u32,
                            ); 
                            println!("Test: {:?}", edge_tuple);
                            println!("Test: {:?}", edge_tuple);
                        }
                    }
                }
            }
        }
    }    

    // Create the X skirt
    let skirt_base_x = generate_skirt(&model.vertices, &duplicate_vertices, veclib::Vector3::new(0, 1, 0), model.vertices.len() as u32, data, veclib::Vector3::new(-1.0, 0.0, 0.0), transform_x_local, get_local_data_x, 0, false);
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
    model = model.combine_smart(&skirt_base_x);
    //model = model.combine_smart(&skirt_y);
    //model = model.combine_smart(&skirt_z);
    // Return the model
    model
}
// Generate a skirt from the data and using a slice index and a custom function that will map the two indexed values to their corresponding vector coordinates
pub fn generate_skirt(verts: &Vec<veclib::Vector3<f32>>, duplicated_vertices: &HashMap<(u32, u32, u32), u32>, edge_offset: veclib::Vector3::<u32>, vertex_count: u32, data: &Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>, axis: veclib::Vector3<f32>, transform_function: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>, data_function: fn(&Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>, (usize, usize), usize) -> [f32; 4], slice: usize, flip: bool) -> Model {
    /*
        2------3------4
        |             |
        |             |
        1             5
        |             |
        |             |
        0------7------6
    */
    let mut vertices: Vec<veclib::Vector3<f32>> = Vec::new();
    let mut triangles: Vec<u32> = Vec::new();
    for a in 0..CHUNK_SIZE - 2 {
        for b in 0..CHUNK_SIZE - 2 {
            let local_data = data_function(data, (a, b), slice);
            let mut case = 0_u8;
            //  3---2
            //  |   |
            //  |   |
            //  0---1
            case += ((local_data[0] < 0.0) as u8) * 1;
            case += ((local_data[1] < 0.0) as u8) * 2;
            case += ((local_data[2] < 0.0) as u8) * 4;
            case += ((local_data[3] < 0.0) as u8) * 8;
            // Skip the full and empty cases
            if case == 0 || case == 15 {
                continue;
            }
            let offset = veclib::Vector2::<f32>::new(a as f32, b as f32);
            // The vertices to connect
            let tris = super::SQUARES_TRI_TABLE[case as usize];    
            for tri_group in 0..3 {
                let mut tri_global_switched: [u32; 3] = [0; 3];
                let mut hit: bool = false;
                for tri_i in 0..3 {
                    let tri = tris[tri_i+tri_group*3];
                    // Check if the value is negative first
                    if tri != -1 {
                        // The bertex
                        let mut vertex = SQUARES_VERTEX_TABLE[tri as usize];
                        // Interpolation            
                        if vertex == -veclib::Vector2::default_one() {    
                                                  
                              
                            match tri {
                                // TODO: Turn this into a more generalized algorithm
                                1 => {
                                    println!("{} {}", 2 * a, 2 * b);
                                    // First edge, gotta lerp between corner 0 and 1
                                    // This vertex already exists in the main mesh, so no need to duplicate it
                                    let first = transform_function(slice, &SQUARES_VERTEX_TABLE[0], &offset);
                                    let second = transform_function(slice, &SQUARES_VERTEX_TABLE[2], &offset);
                                    let edge_tuple: (u32, u32, u32) = (
                                        0,
                                        first.y() as u32 + second.y() as u32,
                                        first.z() as u32 + second.z() as u32,
                                    );
                                    let value =  inverse_lerp(local_data[0], local_data[3], 0.0);
                                    vertex = SQUARES_VERTEX_TABLE[0].lerp(SQUARES_VERTEX_TABLE[2], value);
                                    println!("Good: {:?}", transform_function(slice, &vertex, &offset));
                                    
                                    //println!("{} {}", 2 * a, 2 * b);
                                    println!("A {:?}", edge_tuple);
                                    tri_global_switched[tri_i] = duplicated_vertices[&(edge_tuple)];         
                                    println!("Bad: {:?}", verts[tri_global_switched[tri_i] as usize]);         
                                }
                                3 => {
                                    /*
                                    // Second edge, gotta lerp between corner 1 and 2
                                    /*
                                    let value =  inverse_lerp(local_data[3], local_data[2], 0.0);
                                    vertex = verts[2].lerp(verts[4], value);
                                    */
                                    */
                                    /*
                                    let edge_tuple: (u32, u32, u32) = (
                                        0,
                                        2 * test_offset.0 as u32 + 2 as u32,
                                        2 * test_offset.1 as u32 + 1 as u32,
                                    );
                                    println!("A {:?}", edge_tuple);
                                    tri_global_switched[tri_i] = duplicated_vertices[&(edge_tuple)];     
                                    */
                                }
                                5 => {
                                    /*
                                    println!("{} {}", 2 * a, 2 * b);
                                    // Third edge, gotta lerp between corner 2 and 3
                                    
                                    let value =  inverse_lerp(local_data[2], local_data[1], 0.0);
                                    vertex = SQUARES_VERTEX_TABLE[4].lerp(SQUARES_VERTEX_TABLE[6], value);                 
                                    println!("Good: {:?}", transform_function(slice, &vertex, &offset));
                                    
                                    
                                    let edge_tuple: (u32, u32, u32) = (
                                        0,
                                        2 * a as u32 + 1 as u32,
                                        2 * b as u32 + 2 as u32,
                                    );
                                    println!("B {:?}", edge_tuple);
                                    tri_global_switched[tri_i] = duplicated_vertices[&(edge_tuple)];                                   
                                    println!("Bad: {:?}", verts[tri_global_switched[tri_i] as usize]);       
                                    */ 
                                }
                                7 => {
                                    /*
                                    // Fourth edge, gotta lerp between corner 3 and 0
                                    /*
                                    let value =  inverse_lerp(local_data[1], local_data[0], 0.0);
                                    vertex = verts[6].lerp(verts[0], value);
                                    */
                                    */
                                    /*
                                    let edge_tuple: (u32, u32, u32) = (
                                        0,
                                        2 * test_offset.0 as u32 + 0 as u32,
                                        2 * test_offset.1 as u32 + 1 as u32,
                                    );
                                    println!("A {:?}", edge_tuple);
                                    tri_global_switched[tri_i] = duplicated_vertices[&(edge_tuple)];
                                    */
                                }
                                _ => {}
                            }                            
                        } else {
                            tri_global_switched[tri_i] = vertices.len() as u32 + vertex_count;
                            // This is a vertex that is not present in the main mesh                    
                            vertices.push(transform_function(slice, &vertex, &offset));
                        }               
                        hit = true;
                    }
                }        
                if hit {
                    // Flip the triangle 
                    if flip {
                        // Swap the first and last indices
                        //tri_global_switched.swap(0, 2);
                    }
                    // Add it
                    triangles.extend(tri_global_switched);
                }
            }
        }
    }
    Model { 
        normals: vec![axis; vertices.len()],
        tangents: vec![veclib::Vector4::default(); vertices.len()],
        uvs: vec![veclib::Vector2::default(); vertices.len()],
        triangles: triangles,
        vertices: vertices, 
    }
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