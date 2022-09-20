use std::mem::{size_of, MaybeUninit};

use crate::{buffer::ArrayBuffer, context::Shared};

use super::{VerticesMut, VerticesRef};

bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct EnabledAttributes: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 1;
        const TANGENTS = 1 << 2;
        const COLORS = 1 << 3;
        const TEX_COORDS = 1 << 4;
    }
}

// This is the maximum number of active attributes that we can have inside a mesh
pub const MAX_MESH_VERTEX_ATTRIBUTES: usize =
    EnabledAttributes::all().bits.trailing_ones() as usize;

// Contains the underlying array buffer for a specific attribute
pub type AttributeBuffer<A> = MaybeUninit<ArrayBuffer<<A as VertexAttribute>::Out>>;

// A named attribute that has a specific name, like "Position", or "Normal"
pub trait VertexAttribute {
    type Out: Shared;

    // Number of elements per attribute
    const COUNT_PER_VERTEX: usize;

    // Inner element type
    const GL_TYPE: u32;

    // Corresponding attribute bitfield
    const ENABLED: EnabledAttributes;

    // Should we normalize the data before we send it?
    const NORMALIZED: bool;

    // Get the proper reference from the wrapper vertex types
    fn from_vertices_ref_as_ref<'a>(vertices: &'a VerticesRef) -> &'a AttributeBuffer<Self>;
    fn from_vertices_mut_as_ref<'a>(vertices: &'a VerticesMut) -> &'a AttributeBuffer<Self>;
    fn from_vertices_mut_as_mut<'a>(vertices: &'a mut VerticesMut)
        -> &'a mut AttributeBuffer<Self>;

    // Insert an attribute buffer into the vertices
    fn insert(vertices: &mut VerticesMut, buffer: ArrayBuffer<Self::Out>);

    // Remove an attribute from the vertices
    fn remove(vertices: &mut VerticesMut);

    // Get the attribute's index
    fn index() -> u32 {
        Self::ENABLED.bits().trailing_zeros()
    }

    // Get the attribute's format
    fn format_any() -> AttributeFormatAny {
        AttributeFormatAny {
            normalized: Self::NORMALIZED,
            stride: size_of::<Self::Out>(),
            enabled: Self::ENABLED,
            attribute_index: Self::index(),
        }
    }
}

// An untyped attribute wrapper that contains all the basic information about attributes
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AttributeFormatAny {
    normalized: bool,
    stride: usize,
    enabled: EnabledAttributes,
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

    // Get the attribute enabled bitfield tag
    pub fn tag(&self) -> EnabledAttributes {
        self.enabled
    }
}

// Macro for easier implementations
use gl::*;
use paste::paste;
macro_rules! impl_vertex_attribute {
    ($attribute:ident, $name:ident, $count:tt, $enabled:ident, $normalized:ident, $out:ident, $gltype:ident) => {
        paste! {
            pub struct $attribute(());

            impl VertexAttribute for $attribute {
                type Out = vek::[<Vec $count>]<$out>;
                const COUNT_PER_VERTEX: usize = $count;
                const GL_TYPE: u32 = $gltype;
                const ENABLED: EnabledAttributes = EnabledAttributes::[<$enabled>];
                const NORMALIZED: bool = $normalized;

                fn from_vertices_ref_as_ref<'a>(vertices: &'a VerticesRef) -> &'a AttributeBuffer<Self> {
                    vertices.$name
                }

                fn from_vertices_mut_as_ref<'a>(vertices: &'a VerticesMut) -> &'a AttributeBuffer<Self> {
                    vertices.$name
                }

                fn from_vertices_mut_as_mut<'a>(vertices: &'a mut VerticesMut) -> &'a mut AttributeBuffer<Self> {
                    vertices.$name
                }

                fn insert(vertices: &mut VerticesMut, buffer: ArrayBuffer<Self::Out>) {
                    if vertices.bitfield.contains(Self::ENABLED) {
                        *vertices.$name = MaybeUninit::new(buffer);
                    } else {
                        vertices.$name.write(buffer);
                    }

                    vertices.bitfield.insert(Self::ENABLED);
                }

                fn remove(vertices: &mut VerticesMut) {
                    vertices.bitfield.remove(Self::ENABLED);
                }
            }
        }
    };
}

// Imeplement the common vertex attributes wrapper types
impl_vertex_attribute!(Position, positions, 3, POSITIONS, false, f32, FLOAT);
impl_vertex_attribute!(Normal, normals, 3, NORMALS, true, i8, BYTE);
impl_vertex_attribute!(Tangent, tangents, 4, TANGENTS, true, i8, BYTE);
impl_vertex_attribute!(Color, colors, 3, COLORS, true, u8, UNSIGNED_BYTE);
impl_vertex_attribute!(TexCoord, uvs, 2, TEX_COORDS, true, u8, UNSIGNED_BYTE);

// All the raw types used by the attributes
pub type VePosition = <Position as VertexAttribute>::Out;
pub type VeNormal = <Normal as VertexAttribute>::Out;
pub type VeTangent = <Tangent as VertexAttribute>::Out;
pub type VeColor = <Color as VertexAttribute>::Out;
pub type VeTexCoord = <TexCoord as VertexAttribute>::Out;
