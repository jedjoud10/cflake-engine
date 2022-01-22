use rendering::basics::model::Model;


use crate::ChunkCoords;
// A custom terrain model
pub struct TModel {
    pub model: Model,
    pub skirts_model: Model,
    pub coords: ChunkCoords,
}
