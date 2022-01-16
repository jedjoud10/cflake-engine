// Export
#![feature(drain_filter)]
pub mod component;
pub mod entity;
mod manager;
pub mod system;
pub use manager::ECSManager;
pub mod utils;
mod tests;