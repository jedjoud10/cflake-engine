use std::mem::MaybeUninit;

use assets::Asset;
use math::AABB;
use obj::TexturedVertex;

use super::{attributes::*};
use crate::{
    buffer::{Buffer, ElementBuffer, ArrayBuffer},
    context::Context, mesh::attributes::RawAttribute,
};

// This specified was buffers / attributes are enabled from within the mesh
bitflags::bitflags! {
    pub struct MeshLayout: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 2;
        const TANGENTS = 1 << 3;
        const COLORS = 1 << 4;
        const TEX_COORD_0 = 1 << 5;
        const ELEMENT_BUFFER_OBJECT = 1 << 6;
    }
}

impl Default for MeshLayout {
    fn default() -> Self {
        Self::empty()
    }
}

// Contains the underlying array buffer for a specific attribute
type AttribBuffer<A: Attribute> = MaybeUninit<ArrayBuffer<A::Out>>;

// A submesh is a collection of 3D vertices connected by triangles (optional)
// Each sub-mesh is associated with a single material
// TODO: Fix missing required attribs like pos
pub struct Mesh {
    // Layout and GL name
    name: u32,
    layout: MeshLayout,

    // Vertex attribute buffers
    pub(super) positions: AttribBuffer<Position>,
    pub(super) normals: AttribBuffer<Normal>,
    pub(super) tangents: AttribBuffer<Tangent>,
    pub(super) colors: AttribBuffer<Color>,
    pub(super) tex_coord_0: AttribBuffer<TexCoord0>,

    // The index buffer (optional) (PS: Supports only triangles rn)
    indices: MaybeUninit<ElementBuffer<u32>>,
}

impl Mesh {
    // Get the vertex attrib layout that we are using
    pub fn layout(&self) -> MeshLayout {
        self.layout
    }

    // Check if the layout contains a feature
    pub fn contains(&self, feature: MeshLayout) -> bool {
        self.layout.contains(feature)
    }

    // Check if we have a vertex attribute that is enabled and active
    pub fn is_attribute_active<T: Attribute>(&self) -> bool {
        self.contains(T::LAYOUT)
    }
    
    // Check if we have an element buffer object that is initialized
    pub fn is_ebo_active(&self) -> bool {
        self.contains(MeshLayout::ELEMENT_BUFFER_OBJECT)
    }

    // Get the underlying index buffer immutably
    pub fn indices(&self) -> Option<&ElementBuffer<u32>> {
        self.is_ebo_active().then(|| unsafe { self.indices.assume_init_ref() })
    }

    // Get the underlying index buffer mutably
    pub fn indices_mut(&mut self) -> Option<&mut ElementBuffer<u32>> {
        self.is_ebo_active().then(|| unsafe { self.indices.assume_init_mut() })
    }

    // Get a vertex attribute buffer immutably
    pub fn attribute_buffer<T: Attribute>(&self) -> Option<&ArrayBuffer<T::Out>> {
        self.is_attribute_active::<T>().then(|| unsafe { T::assume_init_get(self) })
    }

    // Get a vertex attribute buffer mutably
    pub fn attribute_buffer_mut<T: Attribute>(&mut self) -> Option<&mut ArrayBuffer<T::Out>> {
        self.is_attribute_active::<T>().then(|| unsafe { T::assume_init_get_mut(self) })
    }

    // Get the number of vertices that we have in total (this will return None if two or more vectors have different lengths)
    pub fn len(&self) -> Option<usize> {
        /*
        // This function just takes an AttribBuf<T> and returns an Option<usize>
        fn len<T: RawAttribute>(vec: &MaybeUninit<ArrayBuffer<T>>) -> Option<usize> {
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
        */
        todo!()
    }

    

    // Recalculate the vertex normals procedurally based on positions
    pub fn compute_normals(&mut self) {

    }

    // Recalculate the tangents procedurall based on normals, positions, and texture coordinates
    pub fn compute_tangents(&mut self) {

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
impl<'a> Asset<'a> for Mesh {
    type Args = &'a mut Context;

    fn extensions() -> &'static [&'static str] {
        &["obj"]
    }

    fn deserialize(data: assets::Data, args: Self::Args) -> Self {
        // Parse the OBJ mesh into the different vertex attributes and the indices
        let parsed = obj::load_obj::<TexturedVertex, &[u8], u32>(data.bytes()).unwrap();
        let capacity = parsed.vertices.len();

        // Create all the buffers at once
        let mut positions = Vec::with_capacity(capacity);
        let mut normals = Vec::with_capacity(capacity);
        let mut tex_coords_0 = Vec::with_capacity(capacity);
        let indices = parsed.indices;

        // Fill each buffer now
        use vek::{Vec2, Vec3};
        for vertex in parsed.vertices {
            positions.push(Vec3::from_slice(&vertex.position));
            normals.push(Vec3::from_slice(&vertex.normal).map(|f| (f * 127.0) as i8));
            tex_coords_0.push(Vec2::from_slice(&vertex.texture).map(|f| (f * 255.0) as u8));
        }

        // We will now calculate the tangents for each vertex using some math magic
        struct TangentGenerator<'a> {
            // Values read from the mesh
            positions: &'a [vek::Vec3<f32>],
            indices: &'a [u32],
            normals: &'a [vek::Vec3<i8>],
            uvs: &'a [vek::Vec2<u8>],

            // Tangents that we will write to (array is already pre-allocated, so we can just write directly)
            tangents: &'a mut [vek::Vec4<i8>],
        }

        // I love external libraries
        impl<'a> mikktspace::Geometry for TangentGenerator<'a> {
            fn num_faces(&self) -> usize {
                self.indices.len() / 3
            }

            // All the models must be triangulated, so we are gud
            fn num_vertices_of_face(&self, _face: usize) -> usize {
                3
            }

            // Read position using index magic
            fn position(&self, face: usize, vert: usize) -> [f32; 3] {
                let i = self.indices[face * 3 + vert] as usize;
                self.positions[i].into_array()
            }

            // Read normal using index magic
            fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
                let i = self.indices[face * 3 + vert] as usize;
                self.normals[i].map(|x| x as f32 / 127.0).into_array()
            }

            // Read texture coordinate using index magic
            fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
                let i = self.indices[face * 3 + vert] as usize;
                self.uvs[i].map(|x| x as f32 / 255.0).into_array()
            }

            // Write a tangent internally
            fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
                let i = self.indices[face * 3 + vert] as usize;
                self.tangents[i] =
                    vek::Vec4::<f32>::from_slice(&tangent).map(|x| (x * 127.0) as i8);
            }
        }

        // Pre-allocate the tangents and create the mikktspace generator
        let mut tangents = vec![vek::Vec4::<i8>::zero(); capacity];
        let mut gen = TangentGenerator {
            positions: &positions,
            normals: &normals,
            indices: &indices,
            uvs: &tex_coords_0,
            tangents: &mut tangents,
        };

        // Generate the procedural tangents
        assert!(mikktspace::generate_tangents(&mut gen));

        // Set the very sussy bakas (POV: You are slowly going insane)
        todo!()
    }
}