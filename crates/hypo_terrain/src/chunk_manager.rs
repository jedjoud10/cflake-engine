use std::collections::HashMap;

use crate::{ChunkData, chunk_data::ChunkCoords};

// Manages the chunks, makes it easier to do multithreading / compute shader stuff
pub struct ChunkManager {
    pub chunks_to_generate: Vec<(u32, ChunkCoords)>,
    pub chunks: HashMap<veclib::Vector3<i64>, ChunkData>
}

impl ChunkManager {
    // Add a chunk
}