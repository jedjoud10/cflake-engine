use std::{mem::size_of, ptr::null};

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
#[derive(Default)]
struct VertexArrayObject {
    name: u32,
    enabled_attributes_count: u32,
}

// Given a context an a VAO, create an empty attribute buffer
fn generate_attrib_buffer<T: Attribute>(ctx: &mut Context, normalized: bool, vao: &mut VertexArrayObject) -> Buffer<T> {
    let mut buffer = Buffer::<T>::new(ctx, false);

    // Bind the buffer to bind the attributes
    buffer.bind(ctx, gl::ARRAY_BUFFER, |_, _| unsafe {
        // Enable the pointer
        let index = vao.enabled_attributes_count;
        gl::VertexAttribPointer(index, T::COUNT_PER_VERTEX as i32, T::GL_TYPE, normalized.into(), 0, null());
        gl::EnableVertexArrayAttrib(vao.name, index);

        // Increment the counter, since we've enabled the attribute
        vao.enabled_attributes_count += 1;
    });

    buffer
}

// What attributes are enabled in a submesh
bitflags::bitflags! {
    struct SubMeshLayout: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 2;
        const TANGENTS = 1 << 3;
        const COLORS = 1 << 4;
        const TEX_COORD0 = 1 << 5;
        const INDICES = 1 << 6;
    }
}


// A submesh is a collection of 3D vertices connected by triangles
// Each sub-mesh is associated with a single material
pub struct SubMesh {    
    // Vertex attributes
    positions: Buffer<vek::Vec3<f32>>,
    normals: Buffer<vek::Vec3<i8>>,
    tangents: Buffer<vek::Vec4<i8>>,
    colors: Buffer<vek::Rgb<u8>>,
    tex_coord: Buffer<vek::Vec2<u8>>,
    
    // Indices
    indices: Buffer<u32>,
}

impl SubMesh {
    // This creates a new submesh with attribute layout defined by "enabled" and with a specific vertex capacity
    fn new(ctx: &mut Context, enabled: SubMeshLayout, capacity: usize) -> (Self, VertexArrayObject) {
        // Create and bind the VAO, then create a safe VAO wrapper
        let mut vao = unsafe {
            let mut name = 0;
            gl::GenVertexArrays(1, &mut name);
            gl::BindVertexArray(name);
            VertexArrayObject {
                name,
                enabled_attributes_count: 0,
            }
        };

        // Create the sub mesh with empty buffers
        let me = Self {
            positions: generate_attrib_buffer(ctx, false, &mut vao),
            normals: generate_attrib_buffer(ctx, false, &mut vao),
            tangents: generate_attrib_buffer(ctx, false, &mut vao),
            colors: generate_attrib_buffer(ctx, false, &mut vao),
            tex_coord: generate_attrib_buffer(ctx, false, &mut vao),
            indices: generate_attrib_buffer(ctx, false, &mut vao),
        };

        (todo!(), vao)
    }
}

// A mesh is simply a collection of submeshes
pub struct Mesh {
    // Separate vector for storing the VAOs of each submesh
    vaos: Vec<u32>,

    // Each submesh
    shared: Vec<SubMesh>,    
}


impl Mesh {
    // Create a new empty mesh that can be modified later
    fn new(_ctx: &Context) -> Self {
        todo!()
    }

    // Create a mesh from multiple submeshes
    fn from_submeshes(_ctx: &Context, submeshes: Vec<SubMesh>) -> Self {
        todo!()
    }
}

impl Mesh {
    // Add a submesh into the mesh
    // 
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