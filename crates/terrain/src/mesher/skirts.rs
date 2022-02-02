
// Skirt vertex
struct SkirtVertex(veclib::Vector3<f32>);
#[derive(Clone, Copy)]
struct LocalSkirtVertex(veclib::Vector2<f32>);
struct SharedSkirtVertexData {
    normal: veclib::Vector3<f32>,
    color: veclib::Vector3<f32>,
    material_type: u8,
}

// Generate a whole skirt using a specific
pub fn calculate_skirt(
    voxels: &[Voxel],
    interpolation: bool,
    flip: bool,
    chunk_size_factor: f32,
    density_offset: [usize; 4],
    skirts_model: &mut (Model, CustomVertexDataBuffer<u32, u32>),
    indexf: fn(usize, usize, usize) -> usize,
    tf: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>,
) {
    for slice in 0..2 {
        for x in 0..MAIN_CHUNK_SIZE {
            for y in 0..MAIN_CHUNK_SIZE {
                let i = indexf(slice, x, y);
                match calculate_marching_square_case(i, x, y, chunk_size_factor, voxels, interpolation, density_offset) {
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
    chunk_size_factor: f32,
    voxels: &[Voxel],
    interpolation: bool,
    density_offset: [usize; 4],
) -> Option<(u8, veclib::Vector2<f32>, ([Option<LocalSkirtVertex>; 4], SharedSkirtVertexData))> {
    // Get the position
    let p = veclib::Vector2::new(x as f32, y as f32);
    // Get the marching cube case
    let mut case = 0_u8;
    // Keep track of the min density
    let mut min = f32::MAX;
    for j in 0..4 {
        let local_voxel = &voxels[i + density_offset[j]];
        // Increase the case index if we have some voxels that are below the 0.0
        case |= ((local_voxel.density <= 0.0) as u8) << j;
        min = min.min(local_voxel.density);
    }
    let force = min > -3.0 * chunk_size_factor - 30.0;

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
            let n1: veclib::Vector3<f32> = voxel1.normal.into();
            let n2: veclib::Vector3<f32> = voxel2.normal.into();
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
        *local_interpolated_voxel = if (voxel1.density <= 0.0) ^ (voxel2.density <= 0.0) {
            let value: f32 = if interpolation {
                inverse_lerp(voxel1.density, voxel2.density, 0.0)
            } else {
                0.5
            };
            // Get the normal
            let normal = veclib::Vector3::<f32>::lerp(voxel1.normal.into(), voxel2.normal.into(), value);
            // Get the color
            let color = veclib::Vector3::<f32>::lerp(voxel1.color.into(), voxel2.color.into(), value);
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
    Some((
        case,
        p,
        (
            local_interpolated_voxels,
            SharedSkirtVertexData {
                normal: (shared_normal / count as f32) / 128.0,
                color: (shared_color / count as f32) / 255.0,
                material_type: voxels[0].material_type,
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
    model: &mut (Model, CustomVertexDataBuffer<u32, u32>),
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
        model.0.triangles.push(model.0.vertices.len() as u32);
        model.0.vertices.push(vertex.0);
        model.0.normals.push(shared.normal.normalized());
        model.0.colors.push(shared.color);
        model.1.push(shared.material_type as u32);
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
