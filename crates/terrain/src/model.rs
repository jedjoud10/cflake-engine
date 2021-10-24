use std::collections::HashMap;

use rendering::Model;

// A custom terrain model
pub struct TModel {
    pub shader_model_hashmap: HashMap<u8, Model>,
    pub skirt_models: HashMap<u8, Model>,
}
