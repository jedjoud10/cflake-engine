use rendering::basics::Model;
use std::collections::HashMap;

use crate::ChunkCoords;
// A custom terrain model
#[derive(Default)]
pub struct TModel {
    // The sub models and their corresponding material
    pub model: Model,
    pub skirts_model: Model,
    pub coords: ChunkCoords,
}
