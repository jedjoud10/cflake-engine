use crate::buffer::ArrayBuffer;

use super::{VertexAttribute, AttributeBuffer};


// This is implemented for &T only, where T is a vertex attribute
trait VertexAttributeReference<'a>: 'a {
    type Inner: VertexAttribute;

    // Read from a attribute buffer pointer immutably, assuming that it is valid
    unsafe fn assume_init_as_ref(ptr: *const AttributeBuffer<Self::Inner>) -> ArrayBuffer<<Self::Inner as VertexAttribute>::Out>;
}

// This is implemented for &T and &mut T, where T is a vertex attribute
trait MutVertexAttributeReference<'a>: 'a {
    type Inner: VertexAttribute;
    type Ptr: 'static + Copy;

    // Read from a attribute buffer pointer immutably, assuming that it is valid
    unsafe fn assume_init_as_ref(ptr: *const AttributeBuffer<Self::Inner>) -> ArrayBuffer<<Self::Inner as VertexAttribute>::Out>;
}