use rendering::basics::Model;
use std::collections::HashMap;

use crate::ChunkCoords;
// A custom terrain model
pub struct TModel {
    pub model: Model,
    pub skirts_model: Model,
    pub coords: ChunkCoords,
}
