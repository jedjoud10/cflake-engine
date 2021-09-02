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
                    }
                }
            }
        }
    }    
    // Create the X skirt
    let skirt_base_x = generate_skirt(data, transform_x_local, get_local_data_x, 0, false);
    let skirt_end_x = generate_skirt(data, transform_x_local, get_local_data_x, CHUNK_SIZE - 2, true);
    let skirt_x = Model::combine(&skirt_base_x, &skirt_end_x);
    model = model.combine(&skirt_x);
    // Return the model
    model
}
// Generate a skirt from the data and using a slice index and a custom function that will map the two indexed values to their corresponding vector coordinates
pub fn generate_skirt(data: &Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>, transform_function: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>, data_function: fn(&Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>, (usize, usize), usize) -> [f32; 4], slice: usize, flip: bool) -> Model {
    /*
        2------3------4
        |             |
        |             |
        1             5
        |             |
        |             |
        0------7------6
    */
    let mut output_model = Model::default();
    for a in 0..CHUNK_SIZE - 2 {
        for b in 0..CHUNK_SIZE - 2 {
            let local_data = data_function(data, (a, b), slice);
            let local_model = solve_case(local_data, SQUARES_VERTEX_TABLE, veclib::Vector2::<f32>::new(a as f32, b as f32), transform_function, slice, flip);
            output_model = output_model.combine(&local_model);
        }
    }
    output_model
}

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

// Using the local data, solve the marching square case
pub fn solve_case(local_data: [f32; 4], verts: [veclib::Vector2<f32>; 8], offset: veclib::Vector2<f32>, transform_function: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>, slice: usize, flip: bool) -> Model {
    let mut output: Model = Model::default();
    let mut case = 0_u8;
    //  3---2
    //  |   |
    //  |   |
    //  0---1
    case += ((local_data[0] < 0.0) as u8) * 1;
    case += ((local_data[1] < 0.0) as u8) * 2;
    case += ((local_data[2] < 0.0) as u8) * 4;
    case += ((local_data[3] < 0.0) as u8) * 8;
    println!("{:?}", local_data);
    let mut vertices: Vec<veclib::Vector3<f32>> = Vec::new();
    let mut tris_output: Vec<u32> = Vec::new();
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
                let mut vertex = verts[tri as usize];

                // Interpolation            
                if vertex == -veclib::Vector2::default_one() {
                    match tri {
                        // TODO: Turn this into a more generalized algorithm
                        1 => {
                            // First edge, gotta lerp between corner 0 and 1
                            let value =  inverse_lerp(local_data[0], local_data[3], 0.0);
                            vertex = verts[0].lerp(verts[2], value);
                        }
                        3 => {
                            // Second edge, gotta lerp between corner 1 and 2
                            let value =  inverse_lerp(local_data[3], local_data[2], 0.0);
                            vertex = verts[2].lerp(verts[4], value);
                        }
                        5 => {
                            // Third edge, gotta lerp between corner 2 and 3
                            let value =  inverse_lerp(local_data[2], local_data[1], 0.0);
                            vertex = verts[4].lerp(verts[6], value);
                        }
                        7 => {
                            // Fourth edge, gotta lerp between corner 3 and 0
                            let value =  inverse_lerp(local_data[1], local_data[0], 0.0);
                            vertex = verts[6].lerp(verts[0], value);
                        }
                        _ => {}
                    }
                }
                tri_global_switched[tri_i] = vertices.len() as u32;
                vertices.push(transform_function(slice, &vertex, &offset));
                hit = true;
            }
        }        
        if hit {
            // Flip the triangle 
            if flip {
                // Swap the first and last indices
                tri_global_switched.swap(0, 2);
            }
            // Add it
            tris_output.extend(tri_global_switched);
        }
    }
    output.normals = vec![veclib::Vector3::default(); vertices.len()];
    output.tangents = vec![veclib::Vector4::default(); vertices.len()];
    output.uvs = vec![veclib::Vector2::default(); vertices.len()];
    output.vertices = vertices;
    output.triangles = tris_output;
    return output;
}