use std::collections::HashMap;

use rendering::Model;

use crate::Voxel;

// A finalized marching cube case
pub struct TCase {
    pub cube_position: veclib::Vector3<f32>,
    pub leading_voxel: Voxel,
}

// A custom terrain model
pub struct TModel {
    // The sub models and their corresponding material
    pub shader_model_hashmap: HashMap<u8, Model>,
    pub skirt_models: HashMap<u8, Model>,
    // The marching cube cases where we had a surface intersection
    pub intersection_cases: Option<Vec<TCase>>,
}
