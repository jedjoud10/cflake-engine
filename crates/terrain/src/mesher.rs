use super::tables::*;
use super::Voxel;
use super::CHUNK_SIZE;
use rendering::Model;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

// If the average density value is below -AVERAGE_DENSITY_THRESHOLD, then we discard that skirt voxel, if it isn't we can still generate it
const AVERAGE_DENSITY_THRESHOLD: f32 = 3.0;

// Inverse of lerp
fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
    (x - a) / (b - a)
}

// Generate the Marching Cubes model
pub fn generate_model(voxels: &Box<[Voxel]>, size: usize, interpolation: bool, skirts: bool) -> Model {
    let mut model: Model = Model::default();
    let mut skirts_model: Model = Model::default();
    let mut duplicate_vertices: HashMap<(u32, u32, u32), u32> = HashMap::new();
    let mut shared_vertices: Vec<SkirtVertex> = Vec::new();
    // Calculate the density threshold for the skirts
    let density_threshold = AVERAGE_DENSITY_THRESHOLD * (((size as f32)*3.0) / (CHUNK_SIZE-2) as f32);
    // Loop over every voxel
    for x in 0..CHUNK_SIZE - 2 {
        for y in 0..CHUNK_SIZE - 2 {
            for z in 0..CHUNK_SIZE - 2 {
                let i = super::flatten((x, y, z));
                // Calculate the 8 bit number at that voxel position, so get all the 8 neighboring voxels
                let mut case_index = 0u8;
                case_index += ((voxels[i + DATA_OFFSET_TABLE[0]].density > 0.0) as u8) * 1;
                case_index += ((voxels[i + DATA_OFFSET_TABLE[1]].density > 0.0) as u8) * 2;
                case_index += ((voxels[i + DATA_OFFSET_TABLE[2]].density > 0.0) as u8) * 4;
                case_index += ((voxels[i + DATA_OFFSET_TABLE[3]].density > 0.0) as u8) * 8;
                case_index += ((voxels[i + DATA_OFFSET_TABLE[4]].density > 0.0) as u8) * 16;
                case_index += ((voxels[i + DATA_OFFSET_TABLE[5]].density > 0.0) as u8) * 32;
                case_index += ((voxels[i + DATA_OFFSET_TABLE[6]].density > 0.0) as u8) * 64;
                case_index += ((voxels[i + DATA_OFFSET_TABLE[7]].density > 0.0) as u8) * 128;

                // Skip the completely empty and completely filled cases
                if case_index == 0 || case_index == 255 {
                    //continue;
                }
                // Get triangles
                let edges: [i8; 16] = TRI_TABLE[case_index as usize];

                // Local edges for the X axis
                let mut local_edges_x: [(u32, u32, u32); 4] = [(0, 0, 0); 4];
                // Local edges for the X axis
                let mut local_edges_y: [(u32, u32, u32); 4] = [(0, 0, 0); 4];
                // Local edges for the X axis
                let mut local_edges_z: [(u32, u32, u32); 4] = [(0, 0, 0); 4];

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
                        let density1 = voxels[index1].density;
                        let density2 = voxels[index2].density;
                        // Do inverse linear interpolation to find the factor value
                        let value: f32 =  if interpolation { inverse_lerp(density1, density2, 0.0) } else { 0.5 };
                        // Create the vertex
                        let mut vertex = veclib::Vector3::<f32>::lerp(vert1, vert2, value);
                        // Offset the vertex
                        vertex += veclib::Vector3::<f32>::new(x as f32, y as f32, z as f32);
                        let normal: veclib::Vector3<f32> = {
                            let mut normal1 = veclib::Vector3::<f32>::ZERO;
                            let mut normal2 = veclib::Vector3::<f32>::ZERO;

                            // Create the normal
                            normal1.x = (voxels[index1 + DATA_OFFSET_TABLE[3]].density - density1);
                            normal1.y = (voxels[index1 + DATA_OFFSET_TABLE[4]].density - density1);
                            normal1.z = (voxels[index1 + DATA_OFFSET_TABLE[1]].density - density1);
                            normal2.x = (voxels[index2 + DATA_OFFSET_TABLE[3]].density - density2);
                            normal2.y = (voxels[index2 + DATA_OFFSET_TABLE[4]].density - density2);
                            normal2.z = (voxels[index2 + DATA_OFFSET_TABLE[1]].density - density2);
                            veclib::Vector3::<f32>::lerp(normal1, normal2, value)
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
                            model.uvs.push(veclib::Vector2::<f32>::ZERO);
                            model.normals.push(normal.normalized());
                            model.tangents.push(veclib::Vector4::<f32>::ZERO);
                        } else {
                            // The vertex already exists
                            model.triangles.push(duplicate_vertices[&edge_tuple]);
                        }

                        // For the X axis
                        if skirts {
                            if vert1_usize.0 == 0 && vert2_usize.0 == 0 {
                                local_edges_x[MC_EDGES_TO_LOCAL_VERTS_X[edge as usize] as usize] = edge_tuple;
                            }
                            if vert1_usize.0 == CHUNK_SIZE - 2 && vert2_usize.0 == CHUNK_SIZE - 2 && x == CHUNK_SIZE - 3 {
                                local_edges_x[MC_EDGES_TO_LOCAL_VERTS_X[edge as usize] as usize] = edge_tuple;
                            }
                            // For the Y axis
                            if vert1_usize.1 == 0 && vert2_usize.1 == 0 {
                                local_edges_y[MC_EDGES_TO_LOCAL_VERTS_Y[edge as usize] as usize] = edge_tuple;
                            }
                            if vert1_usize.1 == CHUNK_SIZE - 2 && vert2_usize.1 == CHUNK_SIZE - 2 && y == CHUNK_SIZE - 3 {
                                local_edges_y[MC_EDGES_TO_LOCAL_VERTS_Y[edge as usize] as usize] = edge_tuple;
                            }
                            // For the Z axis
                            if vert1_usize.2 == 0 && vert2_usize.2 == 0 {
                                local_edges_z[MC_EDGES_TO_LOCAL_VERTS_Z[edge as usize] as usize] = edge_tuple;
                            }
                            if vert1_usize.2 == CHUNK_SIZE - 2 && vert2_usize.2 == CHUNK_SIZE - 2 && z == CHUNK_SIZE - 3 {
                                local_edges_z[MC_EDGES_TO_LOCAL_VERTS_Z[edge as usize] as usize] = edge_tuple;
                            }
                        }
                    }
                }

                if skirts {
                    // Skirts for the X axis
                    if x == 0 {
                        solve_marching_squares(
                            y,
                            z,
                            i,
                            &voxels,
                            &local_edges_x,
                            density_threshold,
                            &mut shared_vertices,
                            veclib::Vec3Axis::X,
                            0,
                            DENSITY_OFFSET_X,
                            false,
                        );
                    }
                    if x == CHUNK_SIZE - 3 {
                        solve_marching_squares(
                            y,
                            z,
                            super::flatten((x + 1, y, z)),
                            &voxels,
                            &local_edges_x,
                            density_threshold,
                            &mut shared_vertices,
                            veclib::Vec3Axis::X,
                            CHUNK_SIZE - 2,
                            DENSITY_OFFSET_X,
                            true,
                        );
                    }
                    // TODO: Fix the bug where this crashes when the density value doesn't have a decimal
                    // Skirts for the Y axis
                    if y == 0 {
                        solve_marching_squares(
                            x,
                            z,
                            i,
                            &voxels,
                            &local_edges_y,
                            density_threshold,
                            &mut shared_vertices,
                            veclib::Vec3Axis::Y,
                            0,
                            DENSITY_OFFSET_Y,
                            false,
                        );
                    }
                    if y == CHUNK_SIZE - 3 {
                        solve_marching_squares(
                            x,
                            z,
                            super::flatten((x, y + 1, z)),
                            &voxels,
                            &local_edges_y,
                            density_threshold,
                            &mut shared_vertices,
                            veclib::Vec3Axis::Y,
                            CHUNK_SIZE - 2,
                            DENSITY_OFFSET_Y,
                            true,
                        );
                    }

                    // Skirts for the Y axis
                    if z == 0 {
                        solve_marching_squares(
                            y,
                            x,
                            i,
                            &voxels,
                            &local_edges_z,
                            density_threshold,
                            &mut shared_vertices,
                            veclib::Vec3Axis::Z,
                            0,
                            DENSITY_OFFSET_Z,
                            false,
                        );
                    }
                    if z == CHUNK_SIZE - 3 {
                        solve_marching_squares(
                            y,
                            x,
                            super::flatten((x, y, z + 1)),
                            &voxels,
                            &local_edges_z,
                            density_threshold,
                            &mut shared_vertices,
                            veclib::Vec3Axis::Z,
                            CHUNK_SIZE - 2,
                            DENSITY_OFFSET_Z,
                            true,
                        );
                    }
                }
            }
        }
    }

    // Turn the shared vertices into triangle indices
    for shared_vertex in shared_vertices {
        match shared_vertex {
            SkirtVertex::Vertex(vertex, normal) => {
                // This vertex isn't a shared vertex
                skirts_model.triangles.push(skirts_model.vertices.len() as u32 + model.vertices.len() as u32);
                skirts_model.vertices.push(vertex.clone());
                skirts_model.normals.push(normal);
            }
            SkirtVertex::SharedVertex(coord_tuple) => {
                let tri = duplicate_vertices[&coord_tuple];
                // This vertex is a vertex that already exists in the main model
                skirts_model.triangles.push(tri);
            }
        }
    }
    model = model.combine_smart(&skirts_model);
    // Return the model
    model
}

// The type of skirt vertex, normal or shared
pub enum SkirtVertex {
    Vertex(veclib::Vector3<f32>, veclib::Vector3<f32>),
    SharedVertex((u32, u32, u32)),
}

// Solve a single marching squares case using a passed function for
pub fn solve_marching_squares(
    // TODO: Comment deez
    a: usize,
    b: usize,
    i: usize,
    data: &Box<[Voxel]>,
    local_edges: &[(u32, u32, u32); 4],
    density_threshold: f32,
    shared_vertices: &mut Vec<SkirtVertex>,
    axis: veclib::Vec3Axis,
    slice: usize,
    density_offset: [usize; 4],
    flip: bool,
) {
    let mut case = 0_u8;
    // For axis X:
    //  3---2
    //  |   |
    //  |   |
    //  0---1
    let mut local_densities: [f32; 4] = [0.0; 4];

    // Get the local densities
    for j in 0..4 {
        let density = data[i + density_offset[j]].density;
        local_densities[j] = density;
        // Update the case index
        case += ((density <= 0.0) as u8) * 2_u8.pow(j as u32);
    }
    // Get the average density
    let average_density: f32 = local_densities.iter().sum::<f32>() / 4.0;

    // ----This is where the actual mesh generation starts----

    if case == 0 {
        return; /* Always skip if it's empty */
    }
    // Check if it's full and it's out of range of the threshold
    if case == 15 && (average_density < -density_threshold) {
        return;
    }
    let offset = veclib::Vector2::<f32>::new(a as f32, b as f32);
    // The vertices to connect
    let tris = if flip {
        SQUARES_FLIPPED_TRI_TABLE[case as usize]
    } else {
        SQUARES_TRI_TABLE[case as usize]
    };
    // Triangle group, basically just 3 indices
    for tri_group in 0..3 {
        // Each local triangle inside the triangle group
        for tri_i in 0..3 {
            let tri = tris[tri_i + tri_group * 3];
            // Check if the value is negative first
            if tri != -1 {
                // The bertex
                let vertex = SQUARES_VERTEX_TABLE[tri as usize];
                // Interpolation
                if vertex == -veclib::Vector2::ONE {
                    match tri {
                        // TODO: Turn this into a more generalized algorithm
                        1 | 3 | 5 | 7 => {
                            let index = (tri - 1) / 2;
                            let edge_tuple = local_edges[index as usize];
                            // Since I fixed the bug we don't have to worry about the tuple being (0, 0, 0)
                            shared_vertices.push(SkirtVertex::SharedVertex(edge_tuple));
                        }
                        _ => {}
                    }
                } else {
                    // This is a vertex that is not present in the main mesh
                    let new_vertex: veclib::Vector3<f32> = match axis {
                        veclib::Vec3Axis::X => transform_x_local(slice, &vertex, &offset),
                        veclib::Vec3Axis::Y => transform_y_local(slice, &vertex, &offset),
                        veclib::Vec3Axis::Z => transform_z_local(slice, &vertex, &offset),
                    };
                    // Get the normal of the skirt vertex
                    let normal: veclib::Vector3<f32> = match axis {
                        veclib::Vec3Axis::X => veclib::Vector3::<f32>::new(0.0, local_densities[3] - local_densities[0], local_densities[1] - local_densities[0]).normalized(),
                        veclib::Vec3Axis::Y => veclib::Vector3::<f32>::new(local_densities[1] - local_densities[0], 0.0, local_densities[3] - local_densities[0]).normalized(),
                        veclib::Vec3Axis::Z => veclib::Vector3::<f32>::new(local_densities[3] - local_densities[0], local_densities[1] - local_densities[0], 0.0).normalized(),
                    };
                    // Add it
                    shared_vertices.push(SkirtVertex::Vertex(new_vertex, normal));
                }
            }
        }
    }
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the X axis
fn transform_x_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(slice as f32, vertex.y + offset.x, vertex.x + offset.y)
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the Y axis
fn transform_y_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vertex.x + offset.x, slice as f32, vertex.y + offset.y)
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the Z axis
fn transform_z_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vertex.y + offset.y, vertex.x + offset.x, slice as f32)
}
