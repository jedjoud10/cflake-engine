#[cfg(test)]
pub mod test {
    use crate::mesher::builder::SkirtVert;
    use crate::mesher::{INDEX_OFFSET_X, MS_CASE_TO_EDGES, MS_EDGE_TO_VERTICES, SQUARES_VERTEX_TABLE};
    use crate::{unflatten, CHUNK_SIZE};
    pub fn calc_interpolation(d1: f32, d2: f32) -> f32 {
        if true {
            // Inverse of lerp
            -d1 / (d2 - d1)
        } else {
            0.5
        }
    }
    // Calculate a marching square case and it's local voxels
    fn generate_marching_squares_case(voxels: &[f32]) {
        // Get the position
        let _p = veclib::Vector2::new(0.0, 1.0);
        // Get the marching cube case
        let mut case_index = 0_u8;
        for i in 0..4 {
            let density = voxels[0 + INDEX_OFFSET_X[i]];
            // Increase the case index if we have some voxels that are below the 0.0
            case_index |= ((density <= 0.0) as u8) << i;
        }
        dbg!(case_index);
        // Get the interpolated voxels
        // Default half-distance interpolated vertices
        let mut ivertices: [SkirtVert; 4] = [
            SkirtVert::Default(veclib::vec2(0.0, 0.5)),
            SkirtVert::Default(veclib::vec2(0.5, 1.0)),
            SkirtVert::Default(veclib::vec2(1.0, 0.5)),
            SkirtVert::Default(veclib::vec2(0.5, 0.0)),
        ];

        // This is some shared data for this whole
        for edge in MS_CASE_TO_EDGES[case_index as usize] {
            if edge.is_negative() {
                break;
            }
            dbg!(edge);
            // Get the two voxel indices
            let two_voxels = MS_EDGE_TO_VERTICES[edge as usize];
            let index1 = 0 + INDEX_OFFSET_X[two_voxels[0] as usize];
            let index2 = 0 + INDEX_OFFSET_X[two_voxels[1] as usize];
            let value: f32 = calc_interpolation(voxels[index1], voxels[index2]);
            dbg!(value);
            let voxel1_local_position = SQUARES_VERTEX_TABLE[two_voxels[0] as usize];
            let voxel2_local_position = SQUARES_VERTEX_TABLE[two_voxels[1] as usize];
            let position = veclib::Vector2::<f32>::lerp(voxel1_local_position, voxel2_local_position, value);
            ivertices[edge as usize] = SkirtVert::Interpolated(position);
        }
    }
    #[test]
    pub fn test() {
        let mut voxels = vec![0.0; (CHUNK_SIZE + 1).pow(3)];
        for (i, voxel) in voxels.iter_mut().enumerate() {
            *voxel = (unflatten(i).1 + 5) as f32 - 10.0;
        }
        generate_marching_squares_case(&voxels)
    }
}
