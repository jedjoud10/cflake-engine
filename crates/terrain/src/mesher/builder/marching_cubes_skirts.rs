use rendering::basics::model::Model;

use crate::{
    mesher::{
        tables::{MS_CASE_TO_EDGES, MS_CASE_TO_TRIS, MS_EDGE_TO_VERTICES, SQUARES_VERTEX_TABLE},
        MesherSettings, SKIRTS_DIR_FLIP, SKIRTS_DIR_INDEXING_FN, SKIRTS_DIR_INDEX_OFFSET, SKIRTS_DIR_TRANSFORM_FN,
    },
    ChunkCoords, StoredVoxelData, CHUNK_SIZE,
};

// A skirts builder, useful since we can keep track of the current iteration as a field, which organizes somestuff
pub(crate) struct MarchingCubesSkirts {
    settings: MesherSettings,
}

impl MarchingCubesSkirts {
    // Create a new marching cubes skirts builder
    pub fn new(settings: MesherSettings) -> Self {
        Self { settings }
    }
    // Calculate the interpolation value using two densities
    pub fn calc_interpolation(&self, d1: f32, d2: f32) -> f32 {
        //return 0.9;
        if self.settings.interpolation {
            // Inverse of lerp
            -d1 / (d2 - d1)
        } else {
            0.5
        }
    }
    // Generate the marching cubes skirts
    pub fn build(&self, voxels: &StoredVoxelData, _coords: ChunkCoords) -> Model {
        if !self.settings.skirts {
            return Model::default();
        }
        // Model builder data that stores the model along with it's custom vdata
        let mut model = Model::default();
        // Create the skirts in all 3 directions
        for direction in 0..3 {
            // Lookup table for axii directions
            let index_offsets = &SKIRTS_DIR_INDEX_OFFSET[direction];
            let flip = SKIRTS_DIR_FLIP[direction];
            let indexing_function = SKIRTS_DIR_INDEXING_FN[direction];
            let transform_function = SKIRTS_DIR_TRANSFORM_FN[direction];
            // Create the two skirts for this direction
            let mut skirt_settings = SkirtSettings {
                slice_part: false,
                index_offsets,
                flip,
                indexing_function,
                transform_function,
            };
            self.generate_skirt(voxels, &mut model, &skirt_settings);

            // Other side
            skirt_settings.flip = !flip;
            skirt_settings.slice_part = true;
            self.generate_skirt(voxels, &mut model, &skirt_settings);
        }
        model
    }
    // Generate a whole skirt
    fn generate_skirt(&self, voxels: &StoredVoxelData, model: &mut Model, skirt_settings: &SkirtSettings) {
        let slice = (skirt_settings.slice_part as usize) * CHUNK_SIZE;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                // Solve each marching squares case
                let i = (skirt_settings.indexing_function)(skirt_settings.slice_part as usize, x, y);
                // Create some iteration info
                let info = InterInfo {
                    index_offsets: skirt_settings.index_offsets,
                    transform_function: skirt_settings.transform_function,
                    slice,
                    flip: skirt_settings.flip,
                    i,
                    x,
                    y,
                };
                // Generate the case
                if let Some(data) = self.generate_marching_squares_case(voxels, &info) {
                    // And solve the case
                    Self::solve_marching_squares(model, &info, &data)
                }
            }
        }
    }
    // Calculate a marching square case and it's local voxels
    fn generate_marching_squares_case(&self, voxels: &StoredVoxelData, info: &InterInfo) -> Option<SquareData> {
        // Get the position
        let p = veclib::Vector2::new(info.x as f32, info.y as f32);
        // Get the marching cube case
        let mut case_index = 0_u8;
        for i in 0..4 {
            let density = voxels.density(info.i + info.index_offsets[i]);
            // Increase the case index if we have some voxels that are below the 0.0
            case_index |= ((*density <= 0.0) as u8) << i;
        }
        // Exit if this case is invalid
        if case_index == 0 || case_index == 15 {
            return None;
        }

        // Get the interpolated voxels
        // Default half-distance interpolated vertices
        let mut ivertices: [SkirtVert; 4] = [
            SkirtVert::Default(veclib::vec2(0.0, 0.5)),
            SkirtVert::Default(veclib::vec2(0.5, 1.0)),
            SkirtVert::Default(veclib::vec2(1.0, 0.5)),
            SkirtVert::Default(veclib::vec2(0.5, 0.0)),
        ];

        // This is some shared data for this whole
        let mut shared_normal = veclib::Vector3::<f32>::ZERO;
        let mut shared_color = veclib::Vector3::<f32>::ZERO;
        let mut count: usize = 0;
        for edge in MS_CASE_TO_EDGES[case_index as usize] {
            // Exit early
            if edge.is_negative() {
                break;
            }

            // Get the two voxel indices
            let two_voxels = MS_EDGE_TO_VERTICES[edge as usize];
            let index1 = info.i + info.index_offsets[two_voxels[0] as usize];
            let index2 = info.i + info.index_offsets[two_voxels[1] as usize];
            let value: f32 = self.calc_interpolation(*voxels.density(index1), *voxels.density(index2));
            // Now interpolate the voxel attributes
            let normal = veclib::Vector3::<f32>::lerp(
                veclib::Vector3::<f32>::from(*voxels.normal(index1)),
                veclib::Vector3::<f32>::from(*voxels.normal(index2)),
                value,
            )
            .normalized();
            let color = veclib::Vector3::<f32>::lerp(
                veclib::Vector3::<f32>::from(*voxels.color(index1)),
                veclib::Vector3::<f32>::from(*voxels.color(index2)),
                value,
            );

            shared_normal += normal;
            shared_color += color;

            // We must get the local offset of these two voxels
            let voxel1_local_position = SQUARES_VERTEX_TABLE[two_voxels[0] as usize];
            let voxel2_local_position = SQUARES_VERTEX_TABLE[two_voxels[1] as usize];
            let position = veclib::Vector2::<f32>::lerp(voxel1_local_position, voxel2_local_position, value);
            count += 1;
            ivertices[edge as usize] = SkirtVert::Interpolated(position);
        }
        Some(SquareData {
            normal: (shared_normal / count as f32 * 255.0).into(),
            color: (shared_color / count as f32).into(),
            voxel_material: *voxels.voxel_material(info.i),
            position: p,
            case: case_index,
            vertices: ivertices,
        })
    }
    // Solve a single marching squares case using a passed function for transforming the vertex position to world space
    fn solve_marching_squares(model: &mut Model, info: &InterInfo, data: &SquareData) {
        let mut vertices: [veclib::Vector3<f32>; 12] = [veclib::Vector3::ZERO; 12];
        let mut len: usize = 0;
        // Create the triangles from the marching squares case
        let triangles = MS_CASE_TO_TRIS[data.case as usize];
        for (i, triangle) in triangles.chunks(3).enumerate() {
            // Exit early
            if triangle[0].is_negative() {
                break;
            }

            // Add the triangle's vertices
            let new_verts = Self::create_triangle(triangle, info, data);
            vertices[i * 3] = new_verts[0];
            vertices[i * 3 + 1] = new_verts[1];
            vertices[i * 3 + 2] = new_verts[2];
            len += 3;
        }
        // Flip the vertices if needed
        if info.flip {
            for x in 0..(vertices.len() / 3) {
                let swap_index0 = x * 3;
                let swap_index1 = x * 3 + 2;
                vertices.swap(swap_index0, swap_index1);
            }
        }
        // Actually add th
        for (i, vertex) in vertices.into_iter().enumerate() {
            //Exit early
            if i == len {
                return;
            }
            // Vertex builder
            model.triangles.push(model.vertices.len() as u32);
            model
                .vertex_builder()
                .with_position(vertex)
                .with_normal(data.normal)
                .with_color(data.color)
                .with_uv(veclib::Vector2::new(data.voxel_material, 0));
        }
    }
    // Create a marching squares triangle between 3 skirt voxels
    fn create_triangle(indices: &[i8], info: &InterInfo, data: &SquareData) -> [veclib::Vector3<f32>; 3] {
        // Check if the local index is one of the interpolated ones
        let mut vertices = [veclib::Vector3::<f32>::ZERO; 3];
        for (triangle_index, vertex_index) in indices.iter().enumerate() {
            vertices[triangle_index] = if *vertex_index % 2 == 0 {
                // Not interpolated
                let transformed_index = (*vertex_index as usize) / 2;

                (info.transform_function)(info.slice, &SQUARES_VERTEX_TABLE[transformed_index], &data.position)
            } else {
                // Interpolated
                let transformed_index = ((*vertex_index as usize) - 1) / 2;
                //dbg!(&data.vertices);
                let inner = if let &SkirtVert::Interpolated(x) = &data.vertices[transformed_index] {
                    x
                } else {
                    panic!()
                };

                (info.transform_function)(info.slice, &inner, &data.position)
            };
        }
        vertices
    }
}
// Some skirt settings
struct SkirtSettings {
    // Voxel indexing
    slice_part: bool,
    index_offsets: &'static [usize; 4],
    // Meshing
    flip: bool,
    // Functions
    indexing_function: fn(usize, usize, usize) -> usize,
    transform_function: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>,
}
// A single skirt vertex
pub enum SkirtVert {
    Default(veclib::Vector2<f32>),
    Interpolated(veclib::Vector2<f32>),
}
// Skirt vertex group
struct SquareData {
    // Shared voxel data
    normal: veclib::Vector3<i8>,
    color: veclib::Vector3<u8>,
    voxel_material: u8,

    // Meshing data
    position: veclib::Vector2<f32>,
    case: u8,
    vertices: [SkirtVert; 4],
}
// Some information about the current iteration
pub struct InterInfo {
    index_offsets: &'static [usize; 4],
    transform_function: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>,
    slice: usize,
    flip: bool,
    i: usize,
    x: usize,
    y: usize,
}
