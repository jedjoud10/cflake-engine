use crate::ChunkCoords;
use crate::ValidGeneratedVoxelData;
use crate::Voxel;
use crate::MAIN_CHUNK_SIZE;

use super::tables::*;
use ahash::AHashMap;
use half::f16;
use rendering::basics::model::CustomVertexDataBuffer;
use rendering::basics::model::Model;
use std::collections::hash_map::Entry;
use std::ptr::null;
use veclib::vec3;

// Inverse of lerp
fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
    (x - a) / (b - a)
}

// Generate the Marching Cubes model
pub fn generate_model(valid_data: &ValidGeneratedVoxelData, coords: ChunkCoords, interpolation: bool, skirts: bool) -> Model {
    let voxels = &valid_data.voxels;
    let valid_subregions = valid_data.valid_sub_regions;
    let mut duplicate_vertices: AHashMap<(u16, u16, u16), u16> = AHashMap::with_capacity(200);
    let mut model: Model = Model::default();
    let mut materials: CustomVertexDataBuffer<u32, u32> = CustomVertexDataBuffer::<u32, u32>::with_capacity(200, rendering::utils::DataType::U32);    
    let i = std::time::Instant::now();
    // Since we're iterating through every voxel, might as well keep track of the min max densities
    let mut min: f16 = f16::MAX;
    let mut max: f16 = f16::MIN;
    // Loop over every voxel
    for x in 0..MAIN_CHUNK_SIZE {
        for y in 0..MAIN_CHUNK_SIZE {
            for z in 0..MAIN_CHUNK_SIZE {
                let i = super::flatten((x, y, z));
                // If we are not part of a valid subregion, no need to generate
                let current_sub_region = super::flatten_custom((x / (MAIN_CHUNK_SIZE / 2), y / (MAIN_CHUNK_SIZE / 2), z / (MAIN_CHUNK_SIZE / 2)), 2) as u8;
                if valid_subregions >> current_sub_region % 2 == 0 { continue; }
                // Calculate the 8 bit number at that voxel position, so get all the 8 neighboring voxels
                let mut case_index = 0u8;
                // Make sure we have the default submodel/material for this material ID
                for l in 0..8 {
                    let density = voxels[i + DATA_OFFSET_TABLE[l]].density;
                    min = min.min(density);
                    max = max.max(density);
                    case_index |= ((density >= f16::ZERO) as u8) * u8::pow(2, l as u32);
                }          
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
                        let voxel1 = &voxels[index1];
                        let voxel2 = &voxels[index2];
                        // Do inverse linear interpolation to find the factor value
                        let value: f32 = if interpolation {
                            inverse_lerp(voxel1.density.into(), voxel2.density.into(), 0.0_f32).clamp(0.0, 1.0)
                        } else {
                            0.5
                        };
                        // Create the vertex
                        let mut vertex = veclib::Vector3::<f32>::lerp(vert1, vert2, value);
                        // Offset the vertex
                        vertex += veclib::Vector3::<f32>::new(x as f32, y as f32, z as f32);
                        // Get the normal
                        let n1: veclib::Vector3<f32> = vec3(voxel1.normal.x.to_f32(), voxel1.normal.y.to_f32(), voxel1.normal.z.to_f32());
                        let n2: veclib::Vector3<f32> = vec3(voxel2.normal.x.to_f32(), voxel2.normal.y.to_f32(), voxel2.normal.z.to_f32());
                        let normal = veclib::Vector3::<f32>::lerp(n1, n2, value);
                        // Get the color
                        let c1: veclib::Vector3<f32> = voxel1.color.into();
                        let c2: veclib::Vector3<f32> = voxel2.color.into();
                        let mut color = veclib::Vector3::<f32>::lerp(c1, c2, value);
                        color /= 255.0;
                        // The edge tuple used to identify this vertex
                        let edge_tuple: (u16, u16, u16) = (
                            2 * x as u16 + vert1.x as u16 + vert2.x as u16,
                            2 * y as u16 + vert1.y as u16 + vert2.y as u16,
                            2 * z as u16 + vert1.z as u16 + vert2.z as u16,
                        );

                        // Check if this vertex was already added
                        if let Entry::Vacant(e) = duplicate_vertices.entry(edge_tuple) {
                            // Add this vertex
                            e.insert(model.vertices.len() as u16);
                            model.triangles.push(model.vertices.len() as u32);
                            model.vertices.push(vertex);
                            model.normals.push(normal.normalized());
                            model.colors.push(color);
                            materials.push(voxel1.material_type as u32);
                        } else {
                            // The vertex already exists
                            model.triangles.push(duplicate_vertices[&edge_tuple] as u32);
                        }
                    }
                }
            }
        }
    }
    println!("{}ms", i.elapsed().as_millis());
    // Create a completely separate model for skirts
    let mut skirts_model: Model = Model::default();
    let chunk_size_factor = (coords.size / MAIN_CHUNK_SIZE as u64) as f32;
    if skirts {
        // Create the X skirt
        calculate_skirt(
            voxels,
            interpolation,
            false,
            min,
            chunk_size_factor,
            DENSITY_OFFSET_X,
            &mut skirts_model,
            |slice, x, y| super::flatten((slice * (MAIN_CHUNK_SIZE), y, x)),
            transform_x_local,
        );
        // Create the Z skirt
        calculate_skirt(
            voxels,
            interpolation,
            true,
            min,
            chunk_size_factor,
            DENSITY_OFFSET_Z,
            &mut skirts_model,
            |slice, x, y| super::flatten((x, y, slice * (MAIN_CHUNK_SIZE))),
            transform_z_local,
        );
        // Create the Y skirt
        calculate_skirt(
            voxels,
            interpolation,
            true,
            min,
            chunk_size_factor,
            DENSITY_OFFSET_Y,
            &mut skirts_model,
            |slice, x, y| super::flatten((x, slice * (MAIN_CHUNK_SIZE), y)),
            transform_y_local,
        );
    }
    Model::combine(model.with_custom(materials), skirts_model)
}
// Skirt vertex
struct SkirtVertex(veclib::Vector3<f32>);
#[derive(Clone, Copy)]
struct LocalSkirtVertex(veclib::Vector2<f32>);
struct SharedSkirtVertexData {
    normal: veclib::Vector3<f16>,
    color: veclib::Vector3<u8>,
}

// Generate a whole skirt using a specific
pub fn calculate_skirt(
    voxels: &[Voxel],
    interpolation: bool,
    flip: bool,
    global_min: f16,
    chunk_size_factor: f32,
    density_offset: [usize; 4],
    skirts_model: &mut Model,
    indexf: fn(usize, usize, usize) -> usize,
    tf: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>,
) {
    for slice in 0..2 {
        for x in 0..MAIN_CHUNK_SIZE {
            for y in 0..MAIN_CHUNK_SIZE {
                let i = indexf(slice, x, y);
                match calculate_marching_square_case(i, x, y, global_min, chunk_size_factor, voxels, interpolation, density_offset) {
                    Some((case, p, ilv)) => {
                        // We intersected the surface
                        solve_marching_squares(slice * MAIN_CHUNK_SIZE, case, p, &ilv, skirts_model, (slice == 1) ^ flip, tf)
                    }
                    None => { /* Empty */ }
                }
            }
        }
    }
}
// Calculate a marching square case and it's local voxels
fn calculate_marching_square_case(
    i: usize,
    x: usize,
    y: usize,
    global_min: f16,
    chunk_size_factor: f32,
    voxels: &[Voxel],
    interpolation: bool,
    density_offset: [usize; 4],
) -> Option<(u8, veclib::Vector2<f32>, ([Option<LocalSkirtVertex>; 4], SharedSkirtVertexData))> {
    // Get the position
    let p = veclib::Vector2::new(x as f32, y as f32);
    // Get the marching cube case
    let mut case = 0_u8;
    // Get the local voxels
    let mut local_voxels: [*const Voxel; 4] = [null(); 4];
    // Keep track of the min density
    let mut min = f16::MAX;
    for (j, voxel) in local_voxels.iter_mut().enumerate() {
        let local_voxel = &voxels[i + density_offset[j]];
        // Increase the case index if we have some voxels that are below the 0.0
        if local_voxel.density <= f16::ZERO {
            case |= 2_u8.pow(j as u32);
        }
        min = min.min(local_voxel.density);
        *voxel = local_voxel as *const Voxel;
    }
    // If the difference between this skirt voxel's min density and the global min density is below a threshold we can force the generation for this skirt voxel
    let force = (min - global_min).to_f32().abs() > (32.0 * chunk_size_factor);

    // Exit if this case is invalid
    if case == 0 || ((case == 15) && !force) {
        return None;
    }
    // Get the interpolated voxels
    let mut local_interpolated_voxels: [Option<LocalSkirtVertex>; 4] = [None; 4];
    let mut shared_normal = veclib::Vector3::<f32>::ZERO;
    let mut shared_color = veclib::Vector3::<f32>::ZERO;    
    let mut count: usize = 0;
    for (j, local_interpolated_voxel) in local_interpolated_voxels.iter_mut().enumerate() {
        // This is for every edge
        let two_voxels = MS_EDGE_TO_VERTICES[j as usize];
        let voxel1 = &voxels[i + density_offset[two_voxels[0] as usize]];
        let voxel2 = &voxels[i + density_offset[two_voxels[1] as usize]];
        // If we are completely filled we must take the average
        if case == 15 {
            // Get the normal
            let n1: veclib::Vector3<f32> = vec3(voxel1.normal.x.to_f32(), voxel1.normal.y.to_f32(), voxel1.normal.z.to_f32());
            let n2: veclib::Vector3<f32> = vec3(voxel2.normal.x.to_f32(), voxel2.normal.y.to_f32(), voxel2.normal.z.to_f32());
            // Get the color
            let c1: veclib::Vector3<f32> = voxel1.color.into();
            let c2: veclib::Vector3<f32> = voxel2.color.into();
            shared_normal += n1 + n2;
            shared_color += c1 + c2;
            count += 2;
            *local_interpolated_voxel = None;
            continue;
        }
        // Check if the edge is intersecting the surface
        *local_interpolated_voxel = if (voxel1.density <= f16::ZERO) ^ (voxel2.density <= f16::ZERO) {
            let value: f32 = if interpolation {
                inverse_lerp(voxel1.density.to_f32(), voxel2.density.to_f32(), 0.0 as f32)
            } else {
                0.5
            };
            // Get the normal
            let n1: veclib::Vector3<f32> = vec3(voxel1.normal.x.to_f32(), voxel1.normal.y.to_f32(), voxel1.normal.z.to_f32());
            let n2: veclib::Vector3<f32> = vec3(voxel2.normal.x.to_f32(), voxel2.normal.y.to_f32(), voxel2.normal.z.to_f32());
            let normal = veclib::Vector3::<f32>::lerp(n1, n2, value);
            // Get the color
            let c1: veclib::Vector3<f32> = voxel1.color.into();
            let c2: veclib::Vector3<f32> = voxel2.color.into();
            let color = veclib::Vector3::<f32>::lerp(c1, c2, value);
            shared_normal += normal;
            shared_color += color;

            // We must get the local offset of these two voxels
            let voxel1_local_offset = SQUARES_VERTEX_TABLE[two_voxels[0] as usize];
            let voxel2_local_offset = SQUARES_VERTEX_TABLE[two_voxels[1] as usize];
            let offset = veclib::Vector2::<f32>::lerp(voxel1_local_offset, voxel2_local_offset, value);
            count += 1;
            Some(LocalSkirtVertex(offset))
        } else {
            None
        }
    }
    let normal: veclib::Vector3<f16> = vec3(
        f16::from_f32(shared_normal.x / count as f32),
        f16::from_f32(shared_normal.y / count as f32),
        f16::from_f32(shared_normal.z / count as f32),
    );

    Some((
        case,
        p,
        (
            local_interpolated_voxels,
            SharedSkirtVertexData {
                normal,
                color: (shared_color / count as f32).into(),
            },
        ),
    ))
    // Solve the case
}
// Solve a single marching squares case using a passed function for transforming the vertex position to world space
fn solve_marching_squares(
    slice: usize,
    case: u8,
    offset: veclib::Vector2<f32>,
    ilv: &([Option<LocalSkirtVertex>; 4], SharedSkirtVertexData),
    model: &mut Model,
    flip: bool,
    tf: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>,
) {
    // Allocate just enough
    let mut vec = Vec::<SkirtVertex>::with_capacity(6);
    // Create the triangles from the local skirts
    match case {
        1 => create_triangle(slice, offset, ilv, &[0, 7, 1], tf, &mut vec),
        2 => create_triangle(slice, offset, ilv, &[1, 3, 2], tf, &mut vec),
        3 => {
            create_triangle(slice, offset, ilv, &[0, 7, 2], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[2, 7, 3], tf, &mut vec);
        }
        4 => create_triangle(slice, offset, ilv, &[3, 5, 4], tf, &mut vec),
        5 => {
            // Two triangles at the corners
            create_triangle(slice, offset, ilv, &[7, 6, 5], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[1, 3, 2], tf, &mut vec);
            // Middle quad
            create_triangle(slice, offset, ilv, &[1, 7, 3], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[3, 7, 5], tf, &mut vec);
        }
        6 => {
            create_triangle(slice, offset, ilv, &[2, 1, 5], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[2, 5, 4], tf, &mut vec);
        }
        7 => {
            create_triangle(slice, offset, ilv, &[2, 0, 7], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[2, 5, 4], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[2, 7, 5], tf, &mut vec);
        }
        8 => create_triangle(slice, offset, ilv, &[7, 6, 5], tf, &mut vec),
        9 => {
            create_triangle(slice, offset, ilv, &[1, 0, 6], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[6, 5, 1], tf, &mut vec);
        }
        10 => {
            // Two triangles at the corners
            create_triangle(slice, offset, ilv, &[7, 6, 5], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[2, 1, 3], tf, &mut vec);
            // Middle quad
            create_triangle(slice, offset, ilv, &[1, 7, 3], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[3, 7, 5], tf, &mut vec);
        }
        11 => {
            create_triangle(slice, offset, ilv, &[0, 6, 5], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[0, 3, 2], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[0, 5, 3], tf, &mut vec);
        }
        12 => {
            create_triangle(slice, offset, ilv, &[7, 6, 3], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[3, 6, 4], tf, &mut vec);
        }
        13 => {
            create_triangle(slice, offset, ilv, &[6, 4, 3], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[6, 1, 0], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[6, 3, 1], tf, &mut vec);
        }
        14 => {
            create_triangle(slice, offset, ilv, &[4, 7, 6], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[4, 2, 1], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[4, 1, 7], tf, &mut vec);
        }
        15 => {
            create_triangle(slice, offset, ilv, &[0, 6, 2], tf, &mut vec);
            create_triangle(slice, offset, ilv, &[6, 4, 2], tf, &mut vec);
        }
        0 => {
            /* Empty cases */
            return;
        }
        _ => {
            /* Case number is unsuported */
            panic!()
        }
    };
    // Flip the vertices if needed
    if flip {
        for x in 0..(vec.len() / 3) {
            let swap_index0 = x * 3;
            let swap_index1 = x * 3 + 2;
            vec.swap(swap_index0, swap_index1);
        }
    }
    // Actually add the skirt vertices
    let shared = &ilv.1;
    for vertex in vec {
        model.triangles.push(model.vertices.len() as u32);
        model.vertices.push(vertex.0);
        let normal: veclib::Vector3<f32> = vec3(shared.normal.x.to_f32(), shared.normal.y.to_f32(), shared.normal.z.to_f32());
        model.normals.push(normal.normalized());
        let mut color: veclib::Vector3<f32> = shared.color.into();
        color /= 255.0;
        model.colors.push(color);
    }
}
// Create a marching squares triangle between 3 skirt voxels
fn create_triangle(
    slice: usize,
    offset: veclib::Vector2<f32>,
    ilv: &([Option<LocalSkirtVertex>; 4], SharedSkirtVertexData),
    li: &[usize; 3],
    tf: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>,
    vec: &mut Vec<SkirtVertex>,
) {
    // Check if the local index is one of the interpolated ones
    for i in li {
        // Calculate the position and normal
        let vertex = match i {
            1 | 3 | 5 | 7 => {
                // Interpolated
                let transformed_index = (i - 1) / 2;
                let v = (tf)(slice, &ilv.0[transformed_index].as_ref().unwrap().0, &offset);
                v
            }
            0 | 2 | 4 | 6 => {
                // Not interpolated
                let transformed_index = (i) / 2;
                let v = (tf)(slice, &SQUARES_VERTEX_TABLE[transformed_index], &offset);
                v
            }
            _ => {
                /* The bruh funny */
                panic!()
            }
        };
        vec.push(SkirtVertex(vertex))
    }
}
// Tansform the 2D points into their 3D counterpart
fn transform_x_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(slice as f32, vertex.x + offset.y, vertex.y + offset.x)
}
fn transform_y_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vertex.x + offset.x, slice as f32, vertex.y + offset.y)
}
fn transform_z_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vertex.y + offset.x, vertex.x + offset.y, slice as f32)
}
