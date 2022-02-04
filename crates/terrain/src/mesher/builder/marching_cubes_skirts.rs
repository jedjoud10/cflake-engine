use rendering::{basics::model::{CustomVertexDataBuffer, Model}, utils::DataType::U32};

use crate::{StoredVoxelData, mesher::{MesherSettings, tables::{MS_EDGE_TO_VERTICES, MS_CASE_TO_EDGES, SQUARES_VERTEX_TABLE, MS_CASE_TO_TRIS}}, CHUNK_SIZE, ChunkCoords, utils};

// A skirts builder, useful since we can keep track of the current iteration as a field, which organizes somestuff
pub(crate) struct MarchingCubesSkirts<'a> {
    // Main settings
    voxels: &'a StoredVoxelData,
    settings: MesherSettings,
    model: Model,
    custom_vdata: CustomVertexDataBuffer<u32, u32>,
    
    // Fileds that are related to computations that we are doing in the current iteration    
}

impl<'a> MarchingCubesSkirts<'a> {
    // Calculate the interpolation value using two densities
    pub fn calc_interpolation(&self, d1: f32, d2: f32) -> f32 {
        if self.settings.interpolation {
            // Inverse of lerp
            -d1 / (d2 - d1)
        } else {
            0.5
        }
    }
    // Generate the marching cubes skirts
    pub fn build(voxels: &'a StoredVoxelData, settings: MesherSettings, coords: ChunkCoords) -> Model {
        let mut me = Self {
            voxels,
            settings,
            model: Model::default(),
            custom_vdata: CustomVertexDataBuffer::with_capacity(200, U32),
        };
        // Create the skirts in all 3 directions

        // Combine the model's custom vertex data with the model itself
        let mut model = me.model;
        let custom_vdata = me.custom_vdata;
        model.with_custom(custom_vdata)
    }

    // Generate a whole skirt
    pub fn generate_skirt(
        &mut self,
        voxel: &StoredVoxelData,        
        flip: bool, 
        index_offsets: &'static [usize; 4],
        indexing_function: fn(usize, usize, usize) -> usize,
        transform_function: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>,
    ) {
        let slice = (flip as usize) * CHUNK_SIZE;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                // Solve each marching squares case
                let i = (indexing_function)(slice, x, y);
                // Create some iteration info
                let info = MarchingSquaresIterInfo {
                    index_offsets,
                    transform_function,
                    slice,
                    flip,
                    i,
                    x,
                    y,
                };
                // Generate the case
                if let Some(data) = self.generate_marching_squares_case(&info) {
                    // And solve the case
                    self.solve_marching_squares(&info, &data)
                }
            }
        }        
    }
    // Calculate a marching square case and it's local voxels
    fn generate_marching_squares_case(
        &mut self,
        info: &MarchingSquaresIterInfo,
    ) -> Option<MarchingSquareSkirtData> {
        // Get the position
        let p = veclib::Vector2::new(info.x as f32, info.y as f32);
        // Get the marching cube case
        let mut case_index = 0_u8;
        let mut min = f32::MAX;
        for i in 0..4 {
            let density = self.voxels.density(info.i + info.index_offsets[i]);
            // Increase the case index if we have some voxels that are below the 0.0
            case_index |= ((*density <= 0.0) as u8) << i;
            min = min.min(*density);
        }
        // Exit if this case is invalid
        if case_index == 0 || case_index == 15 { return None; }

        // Get the interpolated voxels
        let mut ivertices: [Option<veclib::Vector2<f32>>; 4] = [None; 4];

        // This is some shared data for this whole 
        let mut shared_normal = veclib::Vector3::<f32>::ZERO;
        let mut shared_color = veclib::Vector3::<f32>::ZERO;    
        let mut count: usize = 0;
        for edge in MS_CASE_TO_EDGES[case_index as usize] {
            if edge == -1 { break; }
            // This is for every edge
            let two_voxels = MS_EDGE_TO_VERTICES[edge as usize];
            let index1 = info.i + info.index_offsets[two_voxels[0] as usize];
            let index2 = info.i + info.index_offsets[two_voxels[1] as usize];
            let value: f32 = self.calc_interpolation(*self.voxels.density(index1), *self.voxels.density(index2));
            // Now interpolate the voxel attributes
            let normal = veclib::Vector3::<f32>::lerp(veclib::Vector3::<f32>::from(*self.voxels.normal(index1)), veclib::Vector3::<f32>::from(*self.voxels.normal(index2)), value);
            let color = veclib::Vector3::<f32>::lerp(veclib::Vector3::<f32>::from(*self.voxels.color(index1)), veclib::Vector3::<f32>::from(*self.voxels.color(index2)), value);
            shared_normal += normal;
            shared_color += color;

            // We must get the local offset of these two voxels
            let voxel1_local_position = SQUARES_VERTEX_TABLE[two_voxels[0] as usize];
            let voxel2_local_position = SQUARES_VERTEX_TABLE[two_voxels[1] as usize];
            let position = veclib::Vector2::<f32>::lerp(voxel1_local_position, voxel2_local_position, value);
            count += 1;
            ivertices[edge as usize] = Some(position)            
        }
        Some(MarchingSquareSkirtData {
            normal: shared_normal / count as f32,
            color: shared_color / count as f32,
            material_type: *self.voxels.material_type(info.i),
            position: p,
            case: case_index,
            vertices: ivertices,
        })
    }
    // Solve a single marching squares case using a passed function for transforming the vertex position to world space
    fn solve_marching_squares(
        &mut self,
        info: &MarchingSquaresIterInfo,
        data: &MarchingSquareSkirtData,
    ) {
        let mut vertices: [veclib::Vector3<f32>; 12] = [veclib::Vector3::ZERO; 12];
        let mut len: usize = 0;
        // Create the triangles from the marching squares case
        let triangles = MS_CASE_TO_TRIS[data.case as usize];
        for (i, triangle) in triangles.chunks(3).enumerate() {
            // Exit early
            if triangle[0].is_negative() { break; }

            // Add the triangle's vertices
            let new_verts = self.create_triangle(triangle, info, data);
            vertices[i*3] = new_verts[0];
            vertices[i*3+1] = new_verts[1];
            vertices[i*3+2] = new_verts[2];
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
            if i == len { return; }            
            self.model.triangles.push(self.model.vertices.len() as u32);
            self.model.vertices.push(vertex);
            self.model.normals.push(data.normal.normalized());
            self.model.colors.push(data.color);
            self.custom_vdata.push(data.material_type as u32);
        }
    }
    // Create a marching squares triangle between 3 skirt voxels
    fn create_triangle(
        &self,
        indices: &[i8],
        info: &MarchingSquaresIterInfo,
        data: &MarchingSquareSkirtData,
    ) -> [veclib::Vector3<f32>; 3] {
        // Check if the local index is one of the interpolated ones
        let mut vertices = [veclib::Vector3::<f32>::ZERO; 3];
        for (triangle_index, vertex_index) in indices.iter().enumerate() {
            vertices[triangle_index] = if *vertex_index % 2 == 0 {
                // Not interpolated
                let transformed_index = (*vertex_index as usize) / 2;
                let v = (info.transform_function)(info.slice, &SQUARES_VERTEX_TABLE[transformed_index], &data.position);
                v
            } else {
                // Interpolated
                let transformed_index = ((*vertex_index as usize) - 1) / 2;
                let v = (info.transform_function)(info.slice, &data.vertices[transformed_index].as_ref().unwrap(), &data.position);
                v
            };
        }
        vertices
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
}




// Skirt vertex group
struct MarchingSquareSkirtData {
    // Shared voxel data
    normal: veclib::Vector3<f32>,
    color: veclib::Vector3<f32>,
    material_type: u8,

    // Meshing data
    position: veclib::Vector2<f32>,
    case: u8,
    vertices: [Option<veclib::Vector2<f32>>; 4],
}
struct MarchingSquaresIterInfo {
    index_offsets: &'static [usize; 4],
    transform_function: fn(usize, &veclib::Vector2<f32>, &veclib::Vector2<f32>) -> veclib::Vector3<f32>,
    slice: usize,
    flip: bool,
    i: usize,
    x: usize,
    y: usize,
}