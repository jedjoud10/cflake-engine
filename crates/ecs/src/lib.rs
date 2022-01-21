// Export
#![feature(drain_filter)]
pub mod component;
pub mod entity;
mod manager;
pub mod manager_special;
pub mod system;
pub use manager::ECSManager;
mod tests;
pub mod utils;
