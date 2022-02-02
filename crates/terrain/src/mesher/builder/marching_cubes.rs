use std::collections::hash_map::Entry;
use ahash::AHashMap;
use rendering::basics::model::{Model, CustomVertexDataBuffer};
use crate::{mesher::{settings::MesherSettings, tables::{DATA_OFFSET_TABLE, TRI_TABLE, VERTEX_TABLE, EDGE_TABLE}}, StoredVoxelData, ChunkCoords, CHUNK_SIZE, flatten};

// Struct that contains everything related to the marching cubes mesh generation
pub(crate) struct MarchingCubes {
    settings: MesherSettings,
}

impl MarchingCubes { 
    // Create a new marching cubes builder
    pub fn new(settings: MesherSettings) -> Self { Self { settings } }   
    // Calculate the interpolation value using two densities
    pub fn calc_interpolation(&self, d1: f32, d2: f32) -> f32 {
        if self.settings.interpolation {
            // Inverse of lerp
            -d1 / (d2 - d1)
        } else {
            0.5
        }
    }
    // Generate the Marching Cubes model
    pub fn build(&self, voxels: &StoredVoxelData, coords: ChunkCoords) -> Model {
        let i = std::time::Instant::now();
        // Pre-allocate so we don't allocate more than needed
        let mut duplicate_vertices: AHashMap<(u8, u8, u8), u16> = AHashMap::with_capacity(128);
        let mut model: Model = Model::with_capacity(128);
        let mut materials: CustomVertexDataBuffer<u32, u32> = CustomVertexDataBuffer::<u32, u32>::with_capacity(128, rendering::utils::DataType::U32);  
        // Loop over every voxel
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    // Convert our 32x32x32 position into the 33x33x33 index, since they are different 
                    let i = flatten((x, y, z));
                    // Calculate the 8 bit number at that voxel position, so get all the 8 neighboring voxels
                    let mut case_index = 0u8;
                    for l in 0..8 {
                        let density = *voxels.density(i + DATA_OFFSET_TABLE[l]);
                        case_index |= ((density > 0.0) as u8) << l;
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
                            let index1 = flatten(vert1_usize);
                            let index2 = flatten(vert2_usize);
                            // Do inverse linear interpolation to find the factor value
                            let value = self.calc_interpolation(*voxels.density(index1), *voxels.density(index2));
                            // Create the vertex
                            let mut vertex = veclib::Vector3::<f32>::lerp(vert1, vert2, value);
                            // Offset the vertex
                            vertex += veclib::Vector3::<f32>::new(x as f32, y as f32, z as f32);
                            // Get the normal
                            let n1: veclib::Vector3<f32> = (*voxels.normal(index1)).into();
                            let n2: veclib::Vector3<f32> = (*voxels.normal(index2)).into();
                            let normal = veclib::Vector3::<f32>::lerp(n1, n2, value);
                            // Get the color
                            let c1: veclib::Vector3<f32> = (*voxels.color(index1)).into();
                            let c2: veclib::Vector3<f32> = (*voxels.color(index1)).into();
                            let color = veclib::Vector3::<f32>::lerp(c1, c2, value) / 255.0;
                            // The edge tuple used to identify this vertex
                            let edge_tuple: (u8, u8, u8) = (
                                2 * x as u8 + vert1.x as u8 + vert2.x as u8,
                                2 * y as u8 + vert1.y as u8 + vert2.y as u8,
                                2 * z as u8 + vert1.z as u8 + vert2.z as u8,
                            );

                            // Check if this vertex was already added
                            if let Entry::Vacant(e) = duplicate_vertices.entry(edge_tuple) {
                                // Add this vertex
                                e.insert(model.vertices.len() as u16);
                                model.triangles.push(model.vertices.len() as u32);
                                model.vertices.push(vertex);
                                model.normals.push(normal);
                                model.colors.push(color);
                                materials.push(*voxels.material_type(index1) as u32);
                            } else {
                                // The vertex already exists
                                model.triangles.push(duplicate_vertices[&edge_tuple] as u32);
                            }
                        }
                    }
                }
            }
        }    
        let mut skirts_model_combined = (Model::default(), CustomVertexDataBuffer::<u32, u32>::with_capacity(32, rendering::utils::DataType::U32));
        /*
        // Create a completely separate model for skirts
        let chunk_size_factor = (coords.size / MAIN_CHUNK_SIZE as u64) as f32;
        if skirts {
            // Create the X skirt
            calculate_skirt(
                voxels,
                interpolation,
                false,
                chunk_size_factor,
                DENSITY_OFFSET_X,
                &mut skirts_model_combined,
                |slice, x, y| super::flatten((slice * (MAIN_CHUNK_SIZE), y, x)),
                transform_x_local,
            );
            // Create the Z skirt
            calculate_skirt(
                voxels,
                interpolation,
                true,
                chunk_size_factor,
                DENSITY_OFFSET_Z,
                &mut skirts_model_combined,
                |slice, x, y| super::flatten((x, y, slice * (MAIN_CHUNK_SIZE))),
                transform_z_local,
            );
            // Create the Y skirt
            calculate_skirt(
                voxels,
                interpolation,
                true,
                chunk_size_factor,
                DENSITY_OFFSET_Y,
                &mut skirts_model_combined,
                |slice, x, y| super::flatten((x, slice * (MAIN_CHUNK_SIZE), y)),
                transform_y_local,
            );
        }
        */
        println!("{:.2}ms", i.elapsed().as_secs_f32() * 1000.0);
        let (skirts_model, skirts_model_custom_data) = skirts_model_combined;
        Model::combine(model.with_custom(materials), skirts_model.with_custom(skirts_model_custom_data))
    }
}