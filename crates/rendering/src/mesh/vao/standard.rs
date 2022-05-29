use crate::{
    buffer::{ArrayBuffer, Buffer, BufferMode},
    context::Context,
    object::{Shared, ToGlTarget, ToGlName}, mesh::{VertexLayout, GeometryBuilder, vao::attributes::RawAttribute},
};
use std::{num::NonZeroU32, ptr::null};

use super::attributes::Attribute;


// Multiple OpenGL attribute buffers stored in the same struct that we will use for normal mesh rendering
pub struct StandardAttributeSet {
    // The raw name of the VAO
    name: u32,

    // The actual attribute buffers
    pub(super) positions: Option<ArrayBuffer<vek::Vec3<f32>>>,
    pub(super) normals: Option<ArrayBuffer<vek::Vec3<i8>>>,
    pub(super) tangents: Option<ArrayBuffer<vek::Vec4<i8>>>,
    pub(super) colors: Option<ArrayBuffer<vek::Rgb<u8>>>,

    // Multiple texture coordiantes (TODO)
    pub(super) tex_coord_0: Option<ArrayBuffer<vek::Vec2<u8>>>,

    // The enabled attributes
    layout: VertexLayout,
}


// Temp auxiliary data for generating the vertex attribute buffers
struct AuxBufGen<'a> {
    vao: u32,
    index: &'a mut u32,
    builder: &'a GeometryBuilder,
    ctx: &'a mut Context,
    mode: BufferMode,
}

// Generate a unique attribute buffer given some settings and the corresponding Rust vector from the geometry builder
fn gen<'a, T: Attribute>(aux: &mut AuxBufGen<'a>, normalized: bool) -> Option<ArrayBuffer<T::Out>> {
    aux.builder.get_attribute_vec::<T>().map(|vec| unsafe {
        // Create the array buffer
        let buffer = ArrayBuffer::new(aux.ctx, aux.mode, &vec).unwrap();
        
        // Bind the buffer to bind the attributes
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer.name());

        // Enable the pointer
        gl::VertexAttribPointer(*aux.index, T::Out::COUNT_PER_VERTEX as i32, T::Out::GL_TYPE, normalized.into(), 0, null());
        gl::EnableVertexArrayAttrib(aux.vao, *aux.index);
        
        // Increment the counter, since we've enabled the attribute
        *aux.index += 1;        

        buffer
    })
}

impl StandardAttributeSet {
    // Create a new attribute set using a context, a VAO, buffer access type, and a geometry builder
    pub fn new(ctx: &mut Context, mode: BufferMode, builder: &GeometryBuilder) -> Self {
        // Create and bind the VAO, then create a safe VAO wrapper
        let vao = unsafe {
            let mut name = 0;
            gl::GenVertexArrays(1, &mut name);
            gl::BindVertexArray(name);
            name
        };

        // We do a bit of copying
        let layout = builder.layout();

        // Helper struct to make buffer initializiation a bit easier
        let mut index = 0u32;
        let mut aux = AuxBufGen {
            vao,
            index: &mut index,
            builder,
            ctx,
            mode,
        };

        // Create the set with valid buffers (if they are enabled)
        use super::attributes::named::*;
        Self {
            name: vao,
            positions: gen::<Position>(&mut aux, false),
            normals: gen::<Normal>(&mut aux, true),
            tangents: gen::<Tangent>(&mut aux, true),
            colors: gen::<Color>(&mut aux, false),
            tex_coord_0: gen::<TexCoord0>(&mut aux, false),
            layout,
        }
    }

    // Get the layout that we are using
    pub fn layout(&self) -> VertexLayout {
        self.layout
    }

    // Get the number of vertices that we have in total (this will return None if one or more vectors have different lengths)
    pub fn len(&self) -> Option<usize> {
        // This function just takes an AttribBuf<T> and returns an Option<usize>
        fn len<T: RawAttribute>(vec: &Option<ArrayBuffer<T>>) -> Option<usize> {
            vec.as_ref().map(Buffer::len)
        }

        // Make sure all the lengths (that are valid) be equal to each other
        let arr = [len(&self.positions), len(&self.normals), len(&self.tangents), len(&self.colors), len(&self.tex_coord_0)];
        let first = arr.iter().find(|opt| opt.is_some()).cloned().flatten()?;

        // Iterate and check
        let valid = arr.into_iter().filter_map(|a| a).all(|len| len == first);

        // Trollinnggggg
        valid.then(|| first)
    }

    // Get an immutable attribute buffer from the set
    pub fn get_attribute_buffer<T: Attribute>(&self) -> Option<&ArrayBuffer<T::Out>> {
        T::get(self)
    }

    // Get a mutable attribute buffer from the set
    pub fn get_attribute_buffer_mut<T: Attribute>(&mut self) -> Option<&mut ArrayBuffer<T::Out>> {
        T::get_mut(self)
    }
}

impl ToGlName for StandardAttributeSet {
    fn name(&self) -> u32 {
        self.name
    }
}