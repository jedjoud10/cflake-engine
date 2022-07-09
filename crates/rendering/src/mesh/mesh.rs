use assets::Asset;
use math::AABB;

use super::{attributes::{named::Position, Attribute}, VertexAssembly};
use crate::{
    buffer::{Buffer, ElementBuffer, ArrayBuffer},
    canvas::ToRasterBuffers,
    context::Context, mesh::attributes::RawAttribute,
};

// Specified what attributes are enabled in a vertex set
bitflags::bitflags! {
    pub struct VertexLayout: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 2;
        const TANGENTS = 1 << 3;
        const COLORS = 1 << 4;
        const TEX_COORD_0 = 1 << 5;
    }
}

impl Default for VertexLayout {
    fn default() -> Self {
        Self::empty()
    }
}

// A submesh is a collection of 3D vertices connected by triangles
// Each sub-mesh is associated with a single material
pub struct Mesh {
    // The raw name of the VAO
    name: u32,

    // Vertex attribute buffers
    pub(super) positions: Option<ArrayBuffer<vek::Vec3<f32>>>,
    pub(super) normals: Option<ArrayBuffer<vek::Vec3<i8>>>,
    pub(super) tangents: Option<ArrayBuffer<vek::Vec4<i8>>>,
    pub(super) colors: Option<ArrayBuffer<vek::Rgb<u8>>>,
    pub(super) tex_coord_0: Option<ArrayBuffer<vek::Vec2<u8>>>,

    // The enabled attributes
    layout: VertexLayout,

    // The index buffer (PS: Supports only triangles rn)
    indices: ElementBuffer<u32>,
}

impl Mesh {
    // Get the vertex attrib layout that we are using
    pub fn layout(&self) -> VertexLayout {
        self.layout
    }

    // Get the underlying index buffer immutably
    pub fn indices(&self) -> &ElementBuffer<u32> {
        &self.indices
    }

    // Get the underlying index buffer mutably
    pub fn indices_mut(&mut self) -> &mut ElementBuffer<u32> {
        &mut self.indices
    }

    // Get the number of vertices that we have in total (this will return None if two or more vectors have different lengths)
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

    // Get a vertex attribute buffer immutably
    pub fn attribute_buffer<T: Attribute>(&self) -> Option<&ArrayBuffer<T::Out>> {
        None
    }

    // Get a vertex attribute buffer mutably
    pub fn attribute_buffer_mut<T: Attribute>(&mut self) -> Option<&mut ArrayBuffer<T::Out>> {
        None
    }
}


/*
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

        unsafe {
            Self {
                name: vao,
                positions: gen::<Position>(&mut aux, false),
                normals: gen::<Normal>(&mut aux, true),
                tangents: gen::<Tangent>(&mut aux, true),
                colors: gen::<Color>(&mut aux, true),
                tex_coord_0: gen::<TexCoord0>(&mut aux, true),
                layout,
            }
        }
*/