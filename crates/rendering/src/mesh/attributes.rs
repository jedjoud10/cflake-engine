use std::ptr::null;

use crate::{
    buffer::{ArrayBuffer, Buffer, BufferMode},
    context::Context,
    mesh::{GeometryBuilder, VertexAssembly, VertexLayout},
    object::{Shared, ToGlName},
};

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

    // Get the OpenGL array buffer from a specific attribute set
    fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>>;
    fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>>;

    // Get the Rust vector from a vertex assembly (PS: The vector might be null)
    fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>>;
    fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>>;

    // Insert a vector into an assembly
    fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>);
}

pub mod named {
    use super::*;
    // Named attributes implement for empty structs
    pub struct Position;
    pub struct Normal;
    pub struct Tangent;
    pub struct Color;
    pub struct TexCoord0;

    impl Attribute for Position {
        type Out = vek::Vec3<f32>;
        const LAYOUT: VertexLayout = VertexLayout::POSITIONS;

        fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
            set.positions.as_ref()
        }

        fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
            set.positions.as_mut()
        }

        fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>> {
            assembly.positions.as_ref()
        }

        fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>> {
            assembly.positions.as_mut()
        }

        fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>) {
            assembly.positions = Some(vec);
        }
    }

    impl Attribute for Normal {
        type Out = vek::Vec3<i8>;
        const LAYOUT: VertexLayout = VertexLayout::NORMALS;

        fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
            set.normals.as_ref()
        }

        fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
            set.normals.as_mut()
        }

        fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>> {
            assembly.normals.as_ref()
        }

        fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>> {
            assembly.normals.as_mut()
        }

        fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>) {
            assembly.normals = Some(vec);
        }
    }

    impl Attribute for Tangent {
        type Out = vek::Vec4<i8>;
        const LAYOUT: VertexLayout = VertexLayout::TANGENTS;

        fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
            set.tangents.as_ref()
        }

        fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
            set.tangents.as_mut()
        }

        fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>> {
            assembly.tangents.as_ref()
        }

        fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>> {
            assembly.tangents.as_mut()
        }

        fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>) {
            assembly.tangents = Some(vec);
        }
    }

    impl Attribute for Color {
        type Out = vek::Rgb<u8>;
        const LAYOUT: VertexLayout = VertexLayout::COLORS;

        fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
            set.colors.as_ref()
        }

        fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
            set.colors.as_mut()
        }

        fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>> {
            assembly.colors.as_ref()
        }

        fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>> {
            assembly.colors.as_mut()
        }

        fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>) {
            assembly.colors = Some(vec);
        }
    }

    impl Attribute for TexCoord0 {
        type Out = vek::Vec2<u8>;
        const LAYOUT: VertexLayout = VertexLayout::TEX_COORD_0;

        fn get(set: &AttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
            set.tex_coord_0.as_ref()
        }

        fn get_mut(set: &mut AttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
            set.tex_coord_0.as_mut()
        }

        fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>> {
            assembly.tex_coord_0.as_ref()
        }

        fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>> {
            assembly.tex_coord_0.as_mut()
        }

        fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>) {
            assembly.tex_coord_0 = Some(vec);
        }
    }
}

pub mod output {
    use super::named::*;
    use super::Attribute;
    // Type aliases for the underlying vertex attribute data
    pub type VePos = <Position as Attribute>::Out;
    pub type VeNormal = <Normal as Attribute>::Out;
    pub type VeTangent = <Tangent as Attribute>::Out;
    pub type VeColor = <Color as Attribute>::Out;
    pub type VeTexCoord0 = <TexCoord0 as Attribute>::Out;
}

// Multiple OpenGL attribute buffers stored in the same struct that we will use for normal mesh rendering
pub struct AttributeSet {
    // The raw name of the VAO
    name: u32,

    // The actual attribute buffers
    positions: Option<ArrayBuffer<vek::Vec3<f32>>>,
    normals: Option<ArrayBuffer<vek::Vec3<i8>>>,
    tangents: Option<ArrayBuffer<vek::Vec4<i8>>>,
    colors: Option<ArrayBuffer<vek::Rgb<u8>>>,

    // Multiple texture coordiantes (TODO)
    tex_coord_0: Option<ArrayBuffer<vek::Vec2<u8>>>,

    // The enabled attributes
    layout: VertexLayout,
}

// Temp auxiliary data for generating the vertex attribute buffers
struct AuxBufGen<'a> {
    vao: u32,
    index: &'a mut u32,
    vertices: &'a mut VertexAssembly,
    ctx: &'a mut Context,
    mode: BufferMode,
}

// Generate a unique attribute buffer given some settings and the corresponding Rust vector from the geometry builder
fn gen<'a, T: Attribute>(aux: &mut AuxBufGen<'a>, normalized: bool) -> Option<ArrayBuffer<T::Out>> {
    aux.vertices.get_mut::<T>().map(|vec| unsafe {
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

        buffer
    })
}

impl AttributeSet {
    // Create a new attribute set using a context, a VAO, buffer access type, and a geometry builder
    pub fn new(ctx: &mut Context, mode: BufferMode, mut vertices: VertexAssembly) -> Self {
        // Create and bind the VAO, then create a safe VAO wrapper
        let vao = unsafe {
            let mut name = 0;
            gl::GenVertexArrays(1, &mut name);
            gl::BindVertexArray(name);
            name
        };

        // We do a bit of copying
        let layout = vertices.layout();

        // Helper struct to make buffer initializiation a bit easier
        let mut index = 0u32;
        let mut aux = AuxBufGen {
            vao,
            index: &mut index,
            vertices: &mut vertices,
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
        let arr = [
            len(&self.positions),
            len(&self.normals),
            len(&self.tangents),
            len(&self.colors),
            len(&self.tex_coord_0),
        ];

        let first = arr.iter().find(|opt| opt.is_some()).cloned().flatten()?;
        let valid = arr.into_iter().flatten().all(|len| len == first);
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

impl ToGlName for AttributeSet {
    fn name(&self) -> u32 {
        self.name
    }
}
