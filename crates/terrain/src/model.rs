use std::collections::HashMap;

use rendering::Model;

// A custom terrain model
pub struct TModel {
    pub material_model_hashmap: HashMap<u8, Model>
}