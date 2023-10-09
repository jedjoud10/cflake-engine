#![warn(missing_docs)]

//! Some utilities used internally by cflake engine. Nothing stops users from using these utilities themselves

/// Bitset utilities. I could probably swap these for hibitset or another lightweight bitset crate
pub mod bitset;

/// Storage utitilies with handles and weak handles
pub mod storage;

/// Main plugin that will create the file logger and time manager
pub mod plugin;

mod tests;

/// Time, tick, and delta time utilities
pub mod time;

/// Re-exports everything
pub mod prelude {
    pub use crate::bitset::*;
    pub use crate::storage::*;
    pub use crate::tests::*;
    pub use crate::time::*;
    pub use log;
    pub use pretty_type_name;
    pub use vek;
    pub use rayon;
}

pub use log;
pub use pretty_type_name;
