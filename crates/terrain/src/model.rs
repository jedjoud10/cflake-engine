use rendering::basics::Model;
use std::collections::HashMap;

use crate::ChunkCoords;
// A custom terrain model
#[derive(Default)]
pub struct TModel {
    // The sub models and their corresponding material
    pub model: Option<Model>,
    pub skirts_model: Option<Model>,
    pub coords: ChunkCoords,
}
