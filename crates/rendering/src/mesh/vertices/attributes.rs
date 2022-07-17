use std::mem::MaybeUninit;

use crate::{buffer::ArrayBuffer, mesh::Mesh, object::Shared};

bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct EnabledAttributes: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 1;
        const TANGENTS = 1 << 2;
        const COLORS = 1 << 3;
        const TEX_COORD = 1 << 4;
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

    // Get an immutable reference to the attribute buffer
    fn as_ref(mesh: &Mesh) -> &AttributeBuffer<Self>;

    // Get a mutable reference to the attribute buffer
    fn as_mut(mesh: &mut Mesh) -> &mut AttributeBuffer<Self>;
}

// Mesh vertex attributes type wrappers
pub struct Position(());
pub struct Normal(());
pub struct Tangent(());
pub struct Color(());
pub struct TexCoord(());

// An untyped attribute wrapper that contains all the basic information about attributes
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

// Position (Vec3<f32>) vertex attribute
impl VertexAttribute for Position {
    type Out = vek::Vec3<f32>;
    const COUNT_PER_VERTEX: usize = 3;
    const GL_TYPE: u32 = gl::FLOAT;
    const ENABLED: EnabledAttributes = EnabledAttributes::POSITIONS;
    const NORMALIZED: bool = false;

    fn as_ref(mesh: &Mesh) -> &AttributeBuffer<Self> {
        &mesh.positions
    }

    fn as_mut(mesh: &mut Mesh) -> &mut AttributeBuffer<Self> {
        &mut mesh.positions
    }
}

// Normal (Vec3<i8>) vertex attribute
impl VertexAttribute for Normal {
    type Out = vek::Vec3<i8>;
    const COUNT_PER_VERTEX: usize = 3;
    const GL_TYPE: u32 = gl::BYTE;
    const ENABLED: EnabledAttributes = EnabledAttributes::NORMALS;
    const NORMALIZED: bool = true;

    fn as_ref(mesh: &Mesh) -> &AttributeBuffer<Self> {
        &mesh.normals
    }

    fn as_mut(mesh: &mut Mesh) -> &mut AttributeBuffer<Self> {
        &mut mesh.normals
    }
}

// Tangent (Vec4<i8>) vertex attribute
impl VertexAttribute for Tangent {
    type Out = vek::Vec4<i8>;
    const COUNT_PER_VERTEX: usize = 4;
    const GL_TYPE: u32 = gl::BYTE;
    const ENABLED: EnabledAttributes = EnabledAttributes::TANGENTS;
    const NORMALIZED: bool = true;

    fn as_ref(mesh: &Mesh) -> &AttributeBuffer<Self> {
        &mesh.tangents
    }

    fn as_mut(mesh: &mut Mesh) -> &mut AttributeBuffer<Self> {
        &mut mesh.tangents
    }
}

// Color (Vec3<u8>) vertex attribute
impl VertexAttribute for Color {
    type Out = vek::Vec3<u8>;
    const COUNT_PER_VERTEX: usize = 3;
    const GL_TYPE: u32 = gl::UNSIGNED_BYTE;
    const ENABLED: EnabledAttributes = EnabledAttributes::COLORS;
    const NORMALIZED: bool = true;

    fn as_ref(mesh: &Mesh) -> &AttributeBuffer<Self> {
        &mesh.colors
    }

    fn as_mut(mesh: &mut Mesh) -> &mut AttributeBuffer<Self> {
        &mut mesh.colors
    }
}

// Texture coordinates (Vec2<u8>) vertex attribute
impl VertexAttribute for TexCoord {
    type Out = vek::Vec2<u8>;
    const COUNT_PER_VERTEX: usize = 2;
    const GL_TYPE: u32 = gl::UNSIGNED_BYTE;
    const ENABLED: EnabledAttributes = EnabledAttributes::COLORS;
    const NORMALIZED: bool = true;

    fn as_ref(mesh: &Mesh) -> &AttributeBuffer<Self> {
        &mesh.uvs
    }

    fn as_mut(mesh: &mut Mesh) -> &mut AttributeBuffer<Self> {
        &mut mesh.uvs
    }
}

// All the raw types used by the attributes
pub type VePosition = <Position as VertexAttribute>::Out;
pub type VeNormal = <Normal as VertexAttribute>::Out;
pub type VeTangent = <Tangent as VertexAttribute>::Out;
pub type VeColor = <Color as VertexAttribute>::Out;
pub type VeTexCoord = <TexCoord as VertexAttribute>::Out;
