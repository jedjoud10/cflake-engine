use super::{VertexLayout, GeometryBuilder};
use crate::{
    buffer::{Buffer, GPUSendable, ArrayBuffer, BufferAccess},
    context::Context,
};
use std::{ptr::null, num::NonZeroU32};

// Attribute base that will make up the elements of compound attributes.
pub trait BaseAttribute: GPUSendable {
    const GL_TYPE: u32;
}

// A compound attribute, like a vector (as in vec2, vec3, vec4) that consists of multiple attributes
pub trait Attribute: GPUSendable {
    const GL_TYPE: u32;
    const COUNT_PER_VERTEX: u32;
}

// Base attribute implementaions
impl BaseAttribute for f32 {
    const GL_TYPE: u32 = gl::FLOAT;
}

impl BaseAttribute for i32 {
    const GL_TYPE: u32 = gl::INT;
}

impl BaseAttribute for u32 {
    const GL_TYPE: u32 = gl::UNSIGNED_INT;
}

impl BaseAttribute for i16 {
    const GL_TYPE: u32 = gl::SHORT;
}

impl BaseAttribute for u16 {
    const GL_TYPE: u32 = gl::UNSIGNED_SHORT;
}

impl BaseAttribute for i8 {
    const GL_TYPE: u32 = gl::BYTE;
}

impl BaseAttribute for u8 {
    const GL_TYPE: u32 = gl::UNSIGNED_BYTE;
}

impl<T: BaseAttribute> Attribute for T {
    const GL_TYPE: u32 = <T as BaseAttribute>::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 1;
}

impl<T: BaseAttribute> Attribute for vek::Vec2<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 2;
}

impl<T: BaseAttribute> Attribute for vek::Vec3<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 3;
}

impl<T: BaseAttribute> Attribute for vek::Vec4<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 4;
}

impl<T: BaseAttribute> Attribute for vek::Rgb<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 3;
}

impl<T: BaseAttribute> Attribute for vek::Rgba<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 4;
}

// Attribute buffer that *might* be disabled, or maybe enabled
type AttribBuf<T> = Option<ArrayBuffer<T>>;

// Multiple attributes stored in the same struct
pub struct AttributeSet {
    // The actual attribute buffers
    positions: AttribBuf<vek::Vec3<f32>>,
    normals: AttribBuf<vek::Vec3<i8>>,
    tangents: AttribBuf<vek::Vec4<i8>>,
    colors: AttribBuf<vek::Rgb<u8>>,

    // Multiple texture coordiantes (TODO)
    tex_coord_0: AttribBuf<vek::Vec2<u8>>,

    // The number of enabled attributes
    count: u32,
}

// A named attribute that has a specific name, like "Position", or "Normal"
pub trait NamedAttribute {
    type Out: GPUSendable;
    const LAYOUT_ID: VertexLayout;

    // Get the OpenGL array buffer from a specific attribute set
    fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>>;
    fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>>;

    // Set a specific vector inside a geometry builder
    fn insert(builder: &mut GeometryBuilder, vec: Vec<Self::Out>);
}

// Named attributes implement for empty structs
pub struct Position;
pub struct Normal;
pub struct Tangent;
pub struct Color;
pub struct TexCoord0;

impl NamedAttribute for Position {
    type Out = vek::Vec3<f32>;
    const LAYOUT_ID: VertexLayout = VertexLayout::POSITIONS;

    fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
        set.positions.as_ref()
    }

    fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
        set.positions.as_mut()
    }

    fn insert(builder: &mut GeometryBuilder, vec: Vec<Self::Out>) {
        builder.positions = vec;
    }
}

impl NamedAttribute for Normal {
    type Out = vek::Vec3<i8>;
    const LAYOUT_ID: VertexLayout = VertexLayout::NORMALS;

    fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
        set.normals.as_ref()
    }

    fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
        set.normals.as_mut()
    }

    fn insert(builder: &mut GeometryBuilder, vec: Vec<Self::Out>) {
        builder.normals = vec;
    }
}

impl NamedAttribute for Tangent {
    type Out = vek::Vec4<i8>;
    const LAYOUT_ID: VertexLayout = VertexLayout::TANGENTS;

    fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
        set.tangents.as_ref()
    }

    fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
        set.tangents.as_mut()
    }

    fn insert(builder: &mut GeometryBuilder, vec: Vec<Self::Out>) {
        builder.tangents = vec;
    }
}

impl NamedAttribute for Color {
    type Out = vek::Rgb<u8>;
    const LAYOUT_ID: VertexLayout = VertexLayout::COLORS;

    fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
        set.colors.as_ref()
    }

    fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
        set.colors.as_mut()
    }

    fn insert(builder: &mut GeometryBuilder, vec: Vec<Self::Out>) {
        builder.colors = vec;
    }
}

impl NamedAttribute for TexCoord0 {
    type Out = vek::Vec2<u8>;
    const LAYOUT_ID: VertexLayout = VertexLayout::TEX_COORD_0;

    fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
        set.tex_coord_0.as_ref()
    }

    fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
        set.tex_coord_0.as_mut()
    }

    fn insert(builder: &mut GeometryBuilder, vec: Vec<Self::Out>) {
        builder.tex_coord_0 = vec;
    }
}

// Type aliases for the underlying vertex attribute data
pub mod vertex {
    use super::*;
    pub type VePos = <Position as NamedAttribute>::Out;
    pub type VeNormal = <Normal as NamedAttribute>::Out;
    pub type VeTangent = <Tangent as NamedAttribute>::Out;
    pub type VeColor = <Color as NamedAttribute>::Out;
    pub type VeTexCoord0 = <TexCoord0 as NamedAttribute>::Out;
}


// Temp auxiliary data for generating the vertex attribute buffers
struct AuxBufGen<'a> {
    vao: NonZeroU32,
    index: &'a mut u32,
    ctx: &'a mut Context,
    access: BufferAccess,
    layout: VertexLayout,
}

// Given a context, layout, target layout and capacity, generate a valid AttribBuf that might be either Some or None
fn gen<'a, T: Attribute>(aux: &mut AuxBufGen<'a>, normalized: bool, target: VertexLayout) -> AttribBuf<T> {
    aux.layout.contains(target).then(|| {
        let mut buffer = ArrayBuffer::<T>::new(aux.ctx, aux.access);

        // Bind the buffer to bind the attributes
        buffer.bind(aux.ctx, |_, _| unsafe {
            // Enable the pointer
            gl::VertexAttribPointer(*aux.index, T::COUNT_PER_VERTEX as i32, T::GL_TYPE, normalized.into(), 0, null());
            gl::EnableVertexArrayAttrib(aux.vao.get(), *aux.index);

            // Increment the counter, since we've enabled the attribute
            *aux.index += 1;
        });

        buffer
    })
}

impl AttributeSet {
    // Create a new attribute set using a context, a VAO, buffer access type, and a geometry builder
    pub(super) fn new(vao: NonZeroU32, ctx: &mut Context, access: BufferAccess, builder: GeometryBuilder) -> Self {
        // Helper struct to make buffer initializiation a bit easier
        let mut index = 0u32;
        let mut aux = AuxBufGen {
            vao,
            index: &mut index,
            ctx,
            access,
            layout,
        };

        // Create the set with valid buffers (if they are enabled)
        Self {
            positions: gen(&mut aux, false, VertexLayout::POSITIONS),
            normals: gen(&mut aux, true, VertexLayout::NORMALS),
            tangents: gen(&mut aux, true, VertexLayout::TANGENTS),
            colors: gen(&mut aux, false, VertexLayout::COLORS),
            tex_coord_0: gen(&mut aux, false, VertexLayout::TEX_COORD_0),
            count: layout.bits().count_ones(),
        }
    }
}
