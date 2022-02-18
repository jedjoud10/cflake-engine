// Export
#![feature(drain_filter)]
#![feature(bool_to_option)]
#![feature(hash_drain_filter)]
#![feature(negative_impls)]
pub mod component;
pub mod entity;
pub mod event;
mod manager;
pub mod system;
pub use manager::ECSManager;
mod tests;
pub mod utils;
pub use rayon;
