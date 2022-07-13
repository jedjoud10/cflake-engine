use std::mem::MaybeUninit;
use crate::{buffer::ArrayBuffer, object::Shared};
use super::{Mesh};

bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct EnabledAttributes: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 1;
        const TANGENTS = 1 << 2;
        const COLORS = 1 << 3;
        const TEX_COORD = 1 << 4;
        const INDICES = 1 << 5;
    }
}

// Contains the underlying array buffer for a specific attribute
pub type AttribBuffer<A> = MaybeUninit<ArrayBuffer<<A as VertexAttribute>::Out>>;


// A named attribute that has a specific name, like "Position", or "Normal"
pub trait VertexAttribute {
    type Out: Shared;

    const COUNT_PER_VERTEX: usize;
    const GL_TYPE: u32;
    const ENABLED: EnabledAttributes;
    const NORMALIZED: bool;

    // Get the immutable and mutable pointers of the attribute's buffer from the mesh
    fn get_ptr(mesh: &Mesh) -> *const MaybeUninit<ArrayBuffer<Self::Out>>;
    fn get_ptr_mut(mesh: &mut Mesh) -> *mut MaybeUninit<ArrayBuffer<Self::Out>>;

    // Calculate the attribute index offset of self
    fn attribute_index() -> u32 {
        Self::ENABLED.bits().trailing_zeros()
    }
}




// This is the maximum number of active attributes that we can have inside a mesh
pub const MAX_MESH_VERTEX_ATTRIBUTES: usize = 5;

// Mesh vertex attributes type wrappers
pub struct Position(());
pub struct Normal(());
pub struct Tangent(());
pub struct Color(());
pub struct TexCoord(());

// An untyped attribute wrapper that contains all the basic information about attributes
// Only used internally for now 
pub struct AttributeFormatAny {
    normalized: bool,
    stride: usize,
    attribute_index: u32,
}

impl AttributeFormatAny {    
    // Get the normalization state of the attribute
    pub fn normalized(&self) -> bool {
        self.normalized
    }
    
    // Get the width of each raw attribute element
    pub fn stride(&self) -> usize {
        self.stride
    }
    
    // Get the final attribute index
    pub fn attribute_index(&self) -> u32 {
        self.attribute_index
    }
}

// Main layout trait that allows us to access the mesh vertices
// This is super useful when we try to insert new vertices into the mesh,
// because we can be sure that the vertex attribute buffers all have the same length
pub trait VertexLayout {
    // The vertex layout represented by the enabled mesh buffers
    const ENABLED: EnabledAttributes;

    // The buffers that we will use when pushing/iterating through vertices
    type Buffers;

    // The owned version of this vertex layout that we will push
    type OwnedIn;

    // Get the buffers from the mesh
    fn fetch(mesh: &mut Mesh) -> Self::Buffers;

    // Push a new vertex into the fetched buffers
    fn push(buffers: &mut Self::Buffers, vertex: Self::OwnedIn);
}

impl VertexLayout for Position {}
impl<A: VertexAttribute> VertexLayout for (Position, A) {}
impl<A: VertexAttribute, B: VertexAttribute> VertexLayout for (Position, A, B) {}