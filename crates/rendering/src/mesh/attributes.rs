use std::{ptr::null, mem::MaybeUninit};

use crate::{
    buffer::{ArrayBuffer},
    object::{Shared, ToGlName},
};

use super::{Mesh, VertexLayout};

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

    // Get the corresponding buffer for this attribute from the mesh
    // This assumes that the underlying buffer is indeed intialized
    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out>;
    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out>;

    // Insert a buffer containing the raw attribute data into a mesh
    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>);

    // This will set the default attribute values for a specific index
    unsafe fn default(index: u32);
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
    
    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.positions.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.positions.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.positions = MaybeUninit::new(buffer);
    }

    unsafe fn default(_index: u32) {
        panic!()
    }    
}

impl Attribute for Normal {
    type Out = vek::Vec3<i8>;
    const LAYOUT: VertexLayout = VertexLayout::NORMALS;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.normals.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.normals.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.normals = MaybeUninit::new(buffer);
    }

    unsafe fn default(index: u32) {
        gl::VertexAttrib4Nbv(index, [127, 127, 127, 0_i8].as_ptr());
    }
}

impl Attribute for Tangent {
    type Out = vek::Vec4<i8>;
    const LAYOUT: VertexLayout = VertexLayout::TANGENTS;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.tangents.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.tangents.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.tangents = MaybeUninit::new(buffer);
    }

    unsafe fn default(index: u32) {
        gl::VertexAttrib4Nbv(index, [0, 0, 0, 127_i8].as_ptr());
    }
}

impl Attribute for Color {
    type Out = vek::Rgb<u8>;
    const LAYOUT: VertexLayout = VertexLayout::COLORS;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.colors.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.colors.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.colors = MaybeUninit::new(buffer);
    }

    unsafe fn default(index: u32) {
        gl::VertexAttrib4Nub(index, 255, 255, 255, 0);
    }
}

impl Attribute for TexCoord0 {
    type Out = vek::Vec2<u8>;
    const LAYOUT: VertexLayout = VertexLayout::TEX_COORD_0;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.tex_coord_0.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.tex_coord_0.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.tex_coord_0 = MaybeUninit::new(buffer);
    }

    unsafe fn default(index: u32) {
        gl::VertexAttrib4Nub(index, 255, 255, 0, 0);
    }
}

// All the raw types used by the attributes
pub type VePos = <Position as Attribute>::Out;
pub type VeNormal = <Normal as Attribute>::Out;
pub type VeTangent = <Tangent as Attribute>::Out;
pub type VeColor = <Color as Attribute>::Out;
pub type VeTexCoord0 = <TexCoord0 as Attribute>::Out;
/*
// Temp auxiliary data for generating the vertex attribute buffers
struct AuxBufGen<'a> {
    vao: u32,
    index: &'a mut u32,
    vertices: &'a mut VertexAssembly,
    ctx: &'a mut Context,
    mode: BufferMode,
}

// Generate a unique attribute buffer given some settings and the corresponding Rust vector from the geometry builder
unsafe fn gen<'a, T: Attribute>(
    aux: &mut AuxBufGen<'a>,
    normalized: bool,
) -> Option<ArrayBuffer<T::Out>> {
    if let Some(vec) = aux.vertices.get_mut::<T>() {
        // Create the array buffer
        let buffer = ArrayBuffer::new(aux.ctx, aux.mode, vec).unwrap();

        // Bind the buffer to bind the attributes
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer.name());

        // Enable the attribute and set it's parameters
        gl::EnableVertexArrayAttrib(aux.vao, *aux.index);
        gl::VertexAttribPointer(
            *aux.index,
            T::Out::COUNT_PER_VERTEX as i32,
            T::Out::GL_TYPE,
            normalized.into(),
            0,
            null(),
        );

        // Increment the counter, since we've enabled the attribute
        *aux.index += 1;
        Some(buffer)
    } else {
        // Set the default values for the missing attribute
        T::default(*aux.index);
        *aux.index += 1;

        // Maidenless?
        None
    }
}
*/