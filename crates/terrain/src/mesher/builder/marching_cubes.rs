use crate::{
    flatten, flatten_vec3,
    mesher::{
        settings::MesherSettings,
        tables::{DATA_OFFSET_TABLE, EDGE_TABLE, TRI_TABLE, VERTEX_TABLE, VERTEX_TABLE_USIZE},
    },
    ChunkCoords, StoredVoxelData, CHUNK_SIZE,
};
use ahash::AHashMap;
use rendering::basics::model::Model;
use std::collections::hash_map::Entry;


// Struct that contains everything related to the marching cubes mesh generation
pub(crate) struct MarchingCubes {
    settings: MesherSettings,
}

impl MarchingCubes {
    // Create a new marching cubes builder
    pub fn new(settings: MesherSettings) -> Self {
        Self { settings }
    }
    // Calculate the interpolation value using two densities
    pub fn calc_interpolation(&self, d1: f32, d2: f32) -> f32 {
        if self.settings.interpolation {
            // Inverse of lerp
            -d1 / (d2 - d1)
        } else {
            0.5
        }
    }
    // Generate the marching cubes case
    fn generate_marching_cubes_case(voxels: &StoredVoxelData, info: &IterInfo) -> u8 {
        // Calculate the 8 bit number at that voxel position, so get all the 8 neighboring voxels
        let mut case_index = 0u8;
        for l in 0..8 {
            let density = *voxels.density(info.i + DATA_OFFSET_TABLE[l]);
            case_index |= ((density > 0.0) as u8) << l;
        }
        case_index
    }
    // Calculate the interpolated vertex data
    fn get_interpolated_vertex(&self, voxels: &StoredVoxelData, info: &IterInfo, edge: EdgeInfo) -> InterpolatedVertexData {
        // Do inverse linear interpolation to find the factor value
        let value = self.calc_interpolation(*voxels.density(edge.index1), *voxels.density(edge.index2));
        // Create the vertex
        let vedge1 = EDGE_TABLE[(edge.index as usize) * 2];
        let vedge2 = EDGE_TABLE[(edge.index as usize) * 2 + 1];
        let mut vertex = veclib::Vector3::<f32>::lerp(VERTEX_TABLE[vedge1], VERTEX_TABLE[vedge2], value);
        // Offset the vertex
        vertex += veclib::Vector3::<f32>::from(info.pos);
        // Get the normal
        let n1: veclib::Vector3<f32> = (*voxels.normal(edge.index1)).into();
        let n2: veclib::Vector3<f32> = (*voxels.normal(edge.index2)).into();
        let normal = veclib::Vector3::<f32>::lerp(n1, n2, value).normalized();
        InterpolatedVertexData { vertex, normal: (normal * 127.0).into() }
    }
    // Solve the marching cubes case and add the vertices to the model
    fn solve_marching_cubes_case(&self, voxels: &StoredVoxelData, model: &mut Model, merger: &mut VertexMerger, info: &IterInfo, data: CubeData) {
        // The vertex indices that are gonna be used for the skirts
        'edge: for edge in TRI_TABLE[data.case as usize] {
            // Make sure the triangle is valid
            if edge.is_negative() {
                break 'edge;
            }
            // Get the vertex in local space
            let vert1 = VERTEX_TABLE_USIZE[EDGE_TABLE[(edge as usize) * 2]];
            let vert2 = VERTEX_TABLE_USIZE[EDGE_TABLE[(edge as usize) * 2 + 1]];
            // The edge tuple used to identify this vertex
            let edge_tuple: (u8, u8, u8) = (
                2 * info.pos.x as u8 + vert1.x as u8 + vert2.x as u8,
                2 * info.pos.y as u8 + vert1.y as u8 + vert2.y as u8,
                2 * info.pos.z as u8 + vert1.z as u8 + vert2.z as u8,
            );

            // Check if this vertex was already added
            if let Entry::Vacant(e) = merger.duplicates.entry(edge_tuple) {
                // Get the interpolated data
                let index1 = flatten_vec3(info.pos + vert1);
                let index2 = flatten_vec3(info.pos + vert2);
                let interpolated = self.get_interpolated_vertex(
                    voxels,
                    info,
                    EdgeInfo {
                        index1,
                        index2,
                        index: edge as usize,
                    },
                );
                // Then add it to the model
                e.insert(model.vertices.len() as u16);
                model.triangles.push(model.vertices.len() as u32);
                model.vertices.push(interpolated.vertex);
                model.normals.push(interpolated.normal);
                model.uvs.push(veclib::Vector2::new(data.voxel_material, 0));
            } else {
                // The vertex already exists
                model.triangles.push(merger.duplicates[&edge_tuple] as u32);
            }
        }
    }
    // Generate the model
    fn generate_model(&self, voxels: &StoredVoxelData, model: &mut Model) {
        // Use vertex merging
        let mut merger = VertexMerger {
            duplicates: AHashMap::with_capacity(64),
        };
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    // Convert our 32x32x32 position into the 33x33x33 index, since they are different
                    let i = flatten((x, y, z));
                    let info = IterInfo { i, pos: veclib::vec3(x, y, z) };

                    // Generate the case index
                    let case = Self::generate_marching_cubes_case(voxels, &info);
                    if case == 0 || case == 255 {
                        continue;
                    }

                    // Then solve it
                    let data = CubeData {
                        voxel_material: *voxels.voxel_material(i),
                        case,
                    };
                    self.solve_marching_cubes_case(voxels, model, &mut merger, &info, data)
                }
            }
        }
    }
    // Generate the Marching Cubes model
    pub fn build(&self, voxels: &StoredVoxelData, _coords: ChunkCoords) -> Model {
        let i = std::time::Instant::now();
        // Create the model data
        let mut model = Model {
            vertices: Vec::with_capacity(5000),
            normals: Vec::with_capacity(5000),
            uvs: Vec::with_capacity(5000),
            triangles: Vec::with_capacity(15000),
            ..Default::default()
        };
        // Then generate the model
        self.generate_model(voxels, &mut model);
        // Combine the model's custom vertex data with the model itself
        println!("Main: {:.2}ms, verts: {}", i.elapsed().as_secs_f32() * 1000.0, model.vertices.len());
        model
    }
}
// Info about the current iteration
struct IterInfo {
    i: usize,
    pos: veclib::Vector3<usize>,
}
// A vertex merger used to tell us when we should merge vertices or not
struct VertexMerger {
    duplicates: AHashMap<(u8, u8, u8), u16>,
}
// Some interpolated vertex data that we calculate for each interesting edge in the marching cube
struct InterpolatedVertexData {
    vertex: veclib::Vector3<f32>,
    normal: veclib::Vector3<i8>,
}
// Edge intersection info
struct EdgeInfo {
    index1: usize,
    index2: usize,
    index: usize,
}
// Info about the marching cube
struct CubeData {
    // Shared voxel data
    voxel_material: u8,

    // Meshing data
    case: u8,
}
