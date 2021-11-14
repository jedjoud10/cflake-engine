use crate::TCase;
use crate::TModel;
use crate::ISOLINE;
use crate::MAIN_CHUNK_SIZE;

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
pub fn generate_model(voxels: &Box<[Voxel]>, _size: usize, interpolation: bool, _skirts: bool) -> TModel {
    let mut duplicate_vertices: HashMap<(u32, u32, u32, u8), u32> = HashMap::new();
    let mut sub_model_hashmap: HashMap<u8, Model> = HashMap::new();
    let mut intersection_cases: Vec<TCase> = Vec::new();
    let i = Instant::now();
    // Loop over every voxel
    for x in 0..MAIN_CHUNK_SIZE {
        for y in 0..MAIN_CHUNK_SIZE {
            for z in 0..MAIN_CHUNK_SIZE {
                let i = super::flatten((x, y, z));
                // Calculate the 8 bit number at that voxel position, so get all the 8 neighboring voxels
                let mut case_index = 0u8;
                // Leading Voxel
                let lv = &voxels[i + DATA_OFFSET_TABLE[0]];

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
                        let voxel1 = &voxels[index1];
                        let voxel2 = &voxels[index2];
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
                        let normal: veclib::Vector3<f32> = veclib::Vector3::<f32>::lerp(voxel1.normal, voxel2.normal, value.clamp(0.0, 1.0));

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
                    leading_voxel: lv.clone(),
                });
            }
        }
    }
    // Create a completely separate model for skirts
    let mut skirts_model: Model = Model::default();
    // Create the X skirt
    for slice in 0..2 {
        for x in 0..MAIN_CHUNK_SIZE {
            for y in 0..MAIN_CHUNK_SIZE {
                let i = super::flatten((slice * (MAIN_CHUNK_SIZE), y, x));
                // Get the position
                let p = veclib::Vector2::new(x as f32, y as f32);
                // Get the marching cube case
                let mut case = 0_u8;
                // Get the local voxels
                //println!("START");
                let local_voxels: Vec<Voxel> = (0..4)
                    .into_iter()
                    .map(|x| {
                        let local_voxel = voxels[i + DENSITY_OFFSET_X[x]];
                        // Increase the case index if we have some voxels that are below the isoline
                        //println!("Le bruh index: {} {}", x, local_voxel.density);
                        if local_voxel.density <= ISOLINE {
                            case |= 2_u8.pow(x as u32);
                        }
                        local_voxel
                    })
                    .collect::<Vec<Voxel>>();
                // Exit if this case is invalid
                if case == 0 || case == 15 {
                    continue;
                }
                let local_voxels: &[Voxel] = &local_voxels[0..4];
                //println!("Case {}", case);
                // Get the interpolated voxels
                let local_interpolated_voxels: Vec<Option<(veclib::Vector3<f32>, veclib::Vector2<f32>)>> = (0..4)
                    .into_iter()
                    .map(|x| {
                        // This is for every edge
                        let two_voxels = MS_EDGE_TO_VERTICES[x as usize];
                        let voxel1 = voxels[i + DENSITY_OFFSET_X[two_voxels[0] as usize]];
                        let voxel2 = voxels[i + DENSITY_OFFSET_X[two_voxels[1] as usize]];
                        // Check if the edge is intersecting the surface
                        if (voxel1.density <= ISOLINE) ^ (voxel2.density <= ISOLINE) {
                            //println!("{} {}", two_voxels[0], two_voxels[1]);
                            //println!("{} {}", voxel1.density, voxel2.density);
                            let value: f32 = if interpolation {
                                inverse_lerp(voxel1.density, voxel2.density, ISOLINE as f32)
                            } else {
                                0.5
                            };
                            // Interpolate between the two voxels
                            let normal = veclib::Vector3::<f32>::lerp(voxel1.normal, voxel2.normal, value);
                            // We must get the local offset of these two voxels
                            let voxel1_local_offset = SQUARES_VERTEX_TABLE[two_voxels[0] as usize];
                            let voxel2_local_offset = SQUARES_VERTEX_TABLE[two_voxels[1] as usize];
                            let offset = veclib::Vector2::<f32>::lerp(voxel1_local_offset, voxel2_local_offset, value);
                            //println!("{}", offset);
                            Some((normal, offset))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Option<(veclib::Vector3<f32>, veclib::Vector2<f32>)>>>();
                let local_interpolated_voxels: &[Option<(veclib::Vector3<f32>, veclib::Vector2<f32>)>] = &local_interpolated_voxels[0..4];
                //println!("{:?}", local_interpolated_voxels);
                // Solve the case
                solve_marching_squares(slice * (MAIN_CHUNK_SIZE), case, p, local_voxels, local_interpolated_voxels, &mut skirts_model, slice == 1);
            }
        }
    }

    // Return the model
    let mut test_hashmap = HashMap::new();
    test_hashmap.insert(0, skirts_model);
    //println!("Elapsed: {}", i.elapsed().as_millis());
    TModel {
        shader_model_hashmap: sub_model_hashmap,
        skirt_models: test_hashmap,
        intersection_cases: Some(intersection_cases),
    }
}

// Skirt vertex
pub struct SkirtVertex {
    pub position: veclib::Vector3<f32>,
    pub normal: veclib::Vector3<f32>,
}

// Funny skirt moment
// 6---5---4
// |       |
// 7       3
// |       |
// 0---1---2

// Solve a single marching squares case using a passed function for
pub fn solve_marching_squares(
    slice: usize,
    case: u8,
    offset: veclib::Vector2<f32>,
    lv: &[Voxel],
    ilv: &[Option<(veclib::Vector3<f32>, veclib::Vector2<f32>)>],
    model: &mut Model,
    flip: bool,
) {
    // Create the triangles from the local skirts
    let mut skirt_vertices = match case {
        1 => create_triangle(slice, offset, lv, ilv, &[0, 7, 1]),
        2 => create_triangle(slice, offset, lv, ilv, &[1, 3, 2]),
        3 => {
            let mut x = create_triangle(slice, offset, lv, ilv, &[0, 7, 2]);
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[2, 7, 3]));
            x
        }
        4 => create_triangle(slice, offset, lv, ilv, &[3, 5, 4]),
        5 => {
            // Two triangles at the corners
            let mut x = create_triangle(slice, offset, lv, ilv, &[7, 6, 5]);
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[1, 3, 2]));
            // Middle quad
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[1, 7, 3]));
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[3, 7, 5]));
            x
        }
        6 => {
            let mut x = create_triangle(slice, offset, lv, ilv, &[2, 1, 5]);
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[2, 5, 4]));
            x
        }
        7 => {
            let mut x = create_triangle(slice, offset, lv, ilv, &[2, 0, 7]);
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[2, 5, 4]));
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[2, 7, 5]));
            x
        }
        8 => create_triangle(slice, offset, lv, ilv, &[7, 6, 5]),
        9 => {
            let mut x = create_triangle(slice, offset, lv, ilv, &[1, 0, 6]);
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[6, 5, 1]));
            x
        }
        10 => {
            // Two triangles at the corners
            let mut x = create_triangle(slice, offset, lv, ilv, &[7, 6, 5]);
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[2, 1, 3]));
            // Middle quad
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[1, 7, 3]));
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[3, 7, 5]));
            x
        }
        11 => {
            let mut x = create_triangle(slice, offset, lv, ilv, &[0, 6, 5]);
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[0, 2, 3]));
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[0, 5, 3]));
            x
        }
        12 => {
            let mut x = create_triangle(slice, offset, lv, ilv, &[7, 6, 3]);
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[3, 6, 4]));
            x
        }
        13 => {
            let mut x = create_triangle(slice, offset, lv, ilv, &[6, 4, 3]);
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[6, 1, 0]));
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[6, 3, 1]));
            x
        }
        14 => {
            let mut x = create_triangle(slice, offset, lv, ilv, &[4, 7, 6]);
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[4, 2, 1]));
            x.append(&mut create_triangle(slice, offset, lv, ilv, &[4, 1, 7]));
            x
        }
        0 | 15 => {
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
        for x in 0..(skirt_vertices.len() / 3) {
            let swap_index0 = x * 3;
            let swap_index1 = x * 3 + 2;
            skirt_vertices.swap(swap_index0, swap_index1);
        }
    }
    // Actually add the skirt vertices
    for x in skirt_vertices {
        model.triangles.push(model.vertices.len() as u32);
        model.vertices.push(x.position);
        model.normals.push(x.normal.normalized());
        model.colors.push(veclib::Vector3::ONE);
    }
}

// Create a marching squares triangle between 3 skirt voxels
pub fn create_triangle(
    slice: usize,
    offset: veclib::Vector2<f32>,
    lv: &[Voxel],
    ilv: &[Option<(veclib::Vector3<f32>, veclib::Vector2<f32>)>],
    li: &[usize; 3],
) -> Vec<SkirtVertex> {
    // Check if the local index is one of the interpolated ones
    let skirt_vertices = li
        .into_iter()
        .map(|i| {
            // Calculate the position and normal
            let (vertex, &normal) = match *i {
                1 | 3 | 5 | 7 => {
                    // Interpolated
                    let transformed_index = (*i - 1) / 2;
                    let v = transform_x_local(slice, &ilv[transformed_index].unwrap().1, &offset);
                    let n = &ilv[transformed_index].as_ref().unwrap().0;
                    (v, n)
                }
                0 | 2 | 4 | 6 => {
                    // Not interpolated
                    let transformed_index = (*i) / 2;
                    let v = transform_x_local(slice, &SQUARES_VERTEX_TABLE[transformed_index], &offset);
                    (v, &lv[transformed_index].normal)
                }
                _ => {
                    /* The bruh funny */
                    panic!()
                }
            };
            // Return
            SkirtVertex { position: vertex, normal: normal }
        })
        .collect::<Vec<SkirtVertex>>();
    skirt_vertices
}
// Transform the local 2D vertex into a 3D vertex with a slice depth based on the X axis
fn transform_x_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(slice as f32, vertex.x + offset.y, vertex.y + offset.x)
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the Y axis
fn transform_y_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vertex.x + offset.x, slice as f32, vertex.y + offset.y)
}

// Transform the local 2D vertex into a 3D vertex with a slice depth based on the Z axis
fn transform_z_local(slice: usize, vertex: &veclib::Vector2<f32>, offset: &veclib::Vector2<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vertex.y + offset.y, vertex.x + offset.x, slice as f32)
}
