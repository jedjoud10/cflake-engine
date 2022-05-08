use std::{mem::size_of, ptr::null, num::NonZeroU32};

use assets::Asset;
use crate::{Buffer, Context, GPUSendable};

// Attribute base that will make up the elements of compound attributes.
trait BaseAttribute: GPUSendable {
    const GL_TYPE: u32;
}

// A compound attribute, like a vector (as in vec2, vec3, vec4) that consists of multiple attributes
trait Attribute: GPUSendable {
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

// The currently stored VAO in the submesh
#[repr(align(32))]
struct VertexArrayObject {
}

// Specified what attributes are enabled in a vertex set
bitflags::bitflags! {
    struct VertexLayout: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 2;
        const TANGENTS = 1 << 3;
        const COLORS = 1 << 4;
        const TEX_COORD_0 = 1 << 5;
    }
}

// Temp auxiliary data for generating the vertex attribute buffers 
struct AuxBufGen<'a> {
    vao: NonZeroU32,
    index: &'a mut u32,
    ctx: &'a mut Context,
    dynamic: bool,
    layout: VertexLayout
}

// Attribute buffer that *might* be disabled, or maybe enabled
type AttribBuf<T> = Option<Buffer<T>>;

// Given a context, layout, target layout and capacity, generate a valid AttribBuf that might be either Some or None
fn gen<'a, T: Attribute>(aux: &mut AuxBufGen<'a>, normalized: bool, target: VertexLayout) -> AttribBuf<T> {
    aux.layout.contains(target).then(|| {
        let mut buffer = Buffer::<T>::new(aux.ctx, !aux.dynamic);

        // Bind the buffer to bind the attributes
        buffer.bind(aux.ctx, gl::ARRAY_BUFFER, |_, _| unsafe {
            // Enable the pointer
            gl::VertexAttribPointer(*aux.index, T::COUNT_PER_VERTEX as i32, T::GL_TYPE, normalized.into(), 0, null());
            gl::EnableVertexArrayAttrib(aux.vao.get(), *aux.index);

            // Increment the counter, since we've enabled the attribute
            *aux.index += 1;
        });

        buffer
    })
}


// A submesh is a collection of 3D vertices connected by triangles
// Each sub-mesh is associated with a single material
pub struct SubMesh {    
    // The VAO that wraps everything up (OpenGL side)
    vao: NonZeroU32,

    // Vertex attributes and the vertex count
    positions: AttribBuf<vek::Vec3<f32>>,
    normals: AttribBuf<vek::Vec3<i8>>,
    tangents: AttribBuf<vek::Vec4<i8>>,
    colors: AttribBuf<vek::Rgb<u8>>,
    tex_coord_0: AttribBuf<vek::Vec2<u8>>,
    vert_count: usize,
    
    // We must always have a valid EBO
    indices: Buffer<u32>,

    // Vertex layout for attributes
    layout: VertexLayout,

    // How many enabled attributes we have
    attributes: u32,
    
    // Can we modify the VAO after we've created it?
    dynamic: bool,
}

impl SubMesh {
    // This creates a new submesh with attribute layout defined by "layout"
    // This will initialize a valid VAO, EBO, and the proper vertex attribute buffers
    fn new(ctx: &mut Context, layout: VertexLayout, dynamic: bool) -> Self {
        // Create and bind the VAO, then create a safe VAO wrapper
        let vao = unsafe {
            let mut name = 0;
            gl::GenVertexArrays(1, &mut name);
            gl::BindVertexArray(name);
            NonZeroU32::new(name).unwrap()
        };

        // Helper struct to make buffer initializiation a bit easier
        let mut index = 0u32;
        let mut aux = AuxBufGen {
            vao,
            index: &mut index,
            ctx,
            dynamic,
            layout,
        };

        // Create the sub mesh with valid buffers (if they are enabled)
        Self {
            vao,
            positions: gen(&mut aux, false, VertexLayout::POSITIONS),
            normals: gen(&mut aux, true, VertexLayout::NORMALS),
            tangents: gen(&mut aux, true, VertexLayout::TANGENTS),
            colors: gen(&mut aux, false, VertexLayout::COLORS),
            tex_coord_0: gen(&mut aux, false, VertexLayout::TEX_COORD_0),            
            indices: Buffer::new(ctx, !dynamic),
            vert_count: 0,
            layout,
            attributes: layout.bits.count_ones(),
            dynamic,
        }
    }

    // Add some vertices, and make sure the layout matches with our own
    /*
    pub fn insert(&mut self, vertices: VertexSet) -> Option<()> {
        None
    }
    */
    // Set the triangles
}

// A mesh is simply a collection of submeshes
pub struct Mesh {
    submeshes: Vec<SubMesh>,    
}


impl Mesh {
    // Create a new empty mesh that can be modified later
    fn new(_ctx: &mut Context) -> Self {
        Self {
            submeshes: Default::default(),
        }
    }

    // Create a mesh from multiple submeshes
    fn from_submeshes(_ctx: &mut Context, submeshes: Vec<SubMesh>) -> Self {    
        Self {
            submeshes,
        }
    }
}

impl Mesh {
    // Add a submesh into the mesh
}

impl Asset for Mesh {
    type OptArgs = ();

    fn is_valid(meta: assets::metadata::AssetMetadata) -> bool {
        meta.extension() == "obj"
    }

    unsafe fn deserialize(bytes: &[u8], args: &Self::OptArgs) -> Self {
        todo!()
    }
}