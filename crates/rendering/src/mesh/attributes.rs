use std::{ptr::null, mem::{MaybeUninit, size_of}, any::TypeId};

use crate::{
    buffer::{ArrayBuffer, BufferMode},
    object::{Shared, ToGlName},
};

use super::{Mesh};

// Attribute base that will make up the elements of compound attributes.
pub trait ScalarAttribute: Shared {
    const GL_TYPE: u32;
}

// A compound attribute, like a vector (as in vec2, vec3, vec4) that consists of multiple attributes
pub trait RawAttribute: Shared {
    const GL_TYPE: u32;
    const COUNT_PER_VERTEX: u32;
}

// Base attribute implementaions
impl ScalarAttribute for f32 {
    const GL_TYPE: u32 = gl::FLOAT;
}

impl ScalarAttribute for i32 {
    const GL_TYPE: u32 = gl::INT;
}

impl ScalarAttribute for u32 {
    const GL_TYPE: u32 = gl::UNSIGNED_INT;
}

impl ScalarAttribute for i16 {
    const GL_TYPE: u32 = gl::SHORT;
}

impl ScalarAttribute for u16 {
    const GL_TYPE: u32 = gl::UNSIGNED_SHORT;
}

impl ScalarAttribute for i8 {
    const GL_TYPE: u32 = gl::BYTE;
}

impl ScalarAttribute for u8 {
    const GL_TYPE: u32 = gl::UNSIGNED_BYTE;
}

impl<T: ScalarAttribute> RawAttribute for T {
    const GL_TYPE: u32 = <T as ScalarAttribute>::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 1;
}

impl<T: ScalarAttribute> RawAttribute for vek::Vec2<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 2;
}

impl<T: ScalarAttribute> RawAttribute for vek::Vec3<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 3;
}

impl<T: ScalarAttribute> RawAttribute for vek::Vec4<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 4;
}

impl<T: ScalarAttribute> RawAttribute for vek::Rgb<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 3;
}

impl<T: ScalarAttribute> RawAttribute for vek::Rgba<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 4;
}

// A named attribute that has a specific name, like "Position", or "Normal"
pub trait Attribute {
    type Out: RawAttribute + Shared;
    const LAYOUT: VertexLayout;
    const NORMALIZED: bool;

    // Get the corresponding buffer for this attribute from the mesh
    // This assumes that the underlying buffer is indeed intialized
    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out>;
    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out>;

    // Insert a buffer containing the raw attribute data into a mesh
    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>);

    // This will set the default attribute values for a specific index
    unsafe fn default();

    // Calculate the attribute index offset of self
    fn attribute_index() -> u32 {
        Self::LAYOUT.bits().trailing_zeros()
    }

    // Calculate the buffer data stride for each raw element of this attribute
    fn stride() -> usize {
        size_of::<Self::Out>()
    }

    // Create a new homogeneous wrapper that contains all the basic information about this attribute
    fn as_attribute_any() -> AttributeFormatAny {
        AttributeFormatAny {
            layout: Self::LAYOUT,
            normalized: Self::NORMALIZED,
            stride: Self::stride(),
            attribute_index: Self::attribute_index(),
        }
    }
}

// An untyped attribute wrapper that contains all the basic information about attributes
// Only used internally for now 
pub struct AttributeFormatAny {
    layout: VertexLayout,
    normalized: bool,
    stride: usize,
    attribute_index: u32,
}

impl AttributeFormatAny {
    // Get the underlying layout of our attribute
    pub fn layout(&self) -> VertexLayout {
        self.layout
    }
    
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

// This is the maximum number of active attributes that we can have inside a mesh
pub const MAX_MESH_VERTEX_ATTRIBUTES: usize = VertexLayout::all().bits().trailing_ones() as usize;

// This specifies what attributes are enabled from within the mesh
bitflags::bitflags! {
    pub struct VertexLayout: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 1;
        const TANGENTS = 1 << 2;
        const COLORS = 1 << 3;
        const TEX_COORD_0 = 1 << 4;
    }
}

impl Default for VertexLayout {
    fn default() -> Self {
        Self::empty()
    }
}

// Position attribute for vertices. Uses Vec3<f32> internally
pub struct Position;

// Normal attribute for vertices. Uses Vec3<i8> internally
pub struct Normal;

// Tangent attribute for vertices. Uses Vec4<i8> internally
pub struct Tangent;

// Color attribute for vertices. Uses Rgba<u8> internally
pub struct Color;

// TexCoord0 (UV) attribute for vertices. Uses Vec2<u8> internally
pub struct TexCoord0;

impl Attribute for Position {
    type Out = vek::Vec3<f32>;
    const LAYOUT: VertexLayout = VertexLayout::POSITIONS;
    const NORMALIZED: bool = false;
    
    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.positions.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.positions.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.positions = MaybeUninit::new(buffer);
    }

    unsafe fn default() {
        panic!()
    }    
}

impl Attribute for Normal {
    type Out = vek::Vec3<i8>;
    const LAYOUT: VertexLayout = VertexLayout::NORMALS;
    const NORMALIZED: bool = true;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.normals.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.normals.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.normals = MaybeUninit::new(buffer);
    }

    unsafe fn default() {
        gl::VertexAttrib4Nbv(Self::attribute_index(), [127, 127, 127, 0_i8].as_ptr());
    }
}

impl Attribute for Tangent {
    type Out = vek::Vec4<i8>;
    const LAYOUT: VertexLayout = VertexLayout::TANGENTS;
    const NORMALIZED: bool= true;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.tangents.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.tangents.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.tangents = MaybeUninit::new(buffer);
    }

    unsafe fn default() {
        gl::VertexAttrib4Nbv(Self::attribute_index(), [0, 0, 0, 127_i8].as_ptr());
    }
}

impl Attribute for Color {
    type Out = vek::Rgb<u8>;
    const LAYOUT: VertexLayout = VertexLayout::COLORS;
    const NORMALIZED: bool = true;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.colors.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.colors.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.colors = MaybeUninit::new(buffer);
    }

    unsafe fn default() {
        gl::VertexAttrib4Nub(Self::attribute_index(), 255, 255, 255, 0);
    }
}

impl Attribute for TexCoord0 {
    type Out = vek::Vec2<u8>;
    const LAYOUT: VertexLayout = VertexLayout::TEX_COORD_0;
    const NORMALIZED: bool = true;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.tex_coord.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.tex_coord.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.tex_coord = MaybeUninit::new(buffer);
    }

    unsafe fn default() {
        gl::VertexAttrib4Nub(Self::attribute_index(), 255, 255, 0, 0);
    }
}

// All the raw types used by the attributes
pub type VePosition = <Position as Attribute>::Out;
pub type VeNormal = <Normal as Attribute>::Out;
pub type VeTangent = <Tangent as Attribute>::Out;
pub type VeColor = <Color as Attribute>::Out;
pub type VeTexCoord0 = <TexCoord0 as Attribute>::Out;