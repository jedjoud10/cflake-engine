// The actual chunk size number that you change
pub const MAIN_CHUNK_SIZE: usize = 32;
// How many voxels in one axis in each chunk?
pub const CHUNK_SIZE: usize = MAIN_CHUNK_SIZE + 2;
// The default LOD factor
pub const DEFAULT_LOD_FACTOR: f32 = 1.8;
// The isoline
pub const ISOLINE: u16 = 32767;
// Export
mod bound_checker;
mod chunk_data;
mod chunk_manager;
mod density;
pub mod mesher;
mod tables;
mod voxel;

pub use bound_checker::*;
pub use chunk_data::*;
pub use chunk_manager::*;
pub use tables::*;
pub use voxel::*;
