use super::CHUNK_SIZE;
use super::tables::*;
use super::Voxel;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use hypo_rendering::Model;

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
    // Inverse of lerp
    fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
        (x - a) / (b - a)
    }
    let skirt_base_x = generate_x_skirt(data, 2);
    model = model.combine(&skirt_base_x);
    // Return the model
    model
}
// Generate the X skirt from the data and using a slice index
pub fn generate_x_skirt(data: &Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>, slice: usize) -> Model {
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
    for y in 0..CHUNK_SIZE - 2 {
        for z in 0..CHUNK_SIZE - 2 {
            let local_data = get_local_data_x(data, (y, z), slice);
            let local_model = solve_case(local_data, SQUARES_VERTEX_TABLE, slice);
            output_model = output_model.combine(&local_model);
        }
    }
    output_model
}

// Get the local data for the X axis using a origin coordinate
pub fn get_local_data_x(data: &Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>, origin: (usize, usize), slice: usize) -> [f32; 4] {
    let local_data: [f32; 4] = [
        data[super::flatten((slice, origin.0, origin.1))].density,
        data[super::flatten((slice, origin.0 + 1, origin.1))].density,
        data[super::flatten((slice, origin.0 + 1, origin.1 + 1))].density,
        data[super::flatten((slice, origin.0, origin.1 + 1))].density       
    ];
    return local_data;
}

// Using the local data, solve the marching square case
pub fn solve_case(local_data: [f32; 4], verts: [veclib::Vector2<f32>; 8], slice: usize) -> Model {
    let mut output: Model = Model::default();
    let mut case = ((local_data[0] > 0.0) as u8) * 1;
    case += ((local_data[1] > 0.0) as u8) * 2;
    case += ((local_data[2] > 0.0) as u8) * 4;
    case += ((local_data[3] > 0.0) as u8) * 8;
    let mut vertices: Vec<veclib::Vector3<f32>> = Vec::new();
    let mut tris_output: Vec<u32> = Vec::new();
    // The vertices to connect
    let tris = super::SQUARES_TRI_TABLE[case as usize];
    for tri in tris {
        // The bertex
        let vertex = verts[tri as usize];
        vertices.push(veclib::Vector3::<f32>::new(vertex.x(), vertex.y(), slice as f32));
        tris_output.push(tris.len() as u32);
    }
    // TODO: Implement linea interpolation here
    output.vertices = vertices;
    output.triangles = tris_output;
    output.normals = Vec::new();
    output.tangents = Vec::new();
    output.uvs = Vec::new();
    return output;
}