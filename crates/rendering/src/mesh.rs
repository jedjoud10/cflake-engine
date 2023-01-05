mod mesh;
pub mod attributes;
pub use attributes::{MeshAttribute, EnabledMeshAttributes, AttributeBuffer, untyped_attributes_from_enabled_attributes};
pub use mesh::*;