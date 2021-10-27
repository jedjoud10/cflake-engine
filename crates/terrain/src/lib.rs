// The actual chunk size number that you change
pub const MAIN_CHUNK_SIZE: usize = 32;
// How many voxels in one axis in each chunk?
pub const CHUNK_SIZE: usize = MAIN_CHUNK_SIZE + 2;
// The default LOD factor
pub const DEFAULT_LOD_FACTOR: f32 = 2.0;
// The isoline
pub const ISOLINE: u16 = 32767;
// Export
mod bound_checker;
mod chunk;
mod chunk_data;
mod chunk_manager;
mod density;
mod detail_manager;
pub mod mesher;
mod model;
mod tables;
mod voxel;

pub use bound_checker::*;
pub use chunk::*;
pub use chunk_data::*;
pub use chunk_manager::*;
pub use detail_manager::*;
pub use model::*;
pub use tables::*;
pub use voxel::*;
