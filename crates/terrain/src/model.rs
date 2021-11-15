use std::collections::HashMap;
use rendering::Model;
// A custom terrain model
#[derive(Default)]
pub struct TModel {
    // The sub models and their corresponding material
    pub models: HashMap<u8, Model>,
    pub skirts_models: HashMap<u8, Model>,
}
