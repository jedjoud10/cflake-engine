use std::{mem::{MaybeUninit, size_of}, ptr::null};

use arrayvec::ArrayVec;
use assets::Asset;
use math::AABB;
use obj::TexturedVertex;

use super::{attributes::*};
use crate::{
    buffer::{Buffer, ElementBuffer, ArrayBuffer, BufferMode},
    context::Context, mesh::attributes::RawAttribute, prelude::Array, object::{ToGlName, Shared},
};

// Contains the underlying array buffer for a specific attribute
type AttribBuffer<A: Attribute> = MaybeUninit<ArrayBuffer<A::Out>>;

// A mesh is a collection of 3D vertices connected by triangles
// Each sub-mesh is associated with a single material
// TODO: Fix buffer assignment without VAO format update
pub struct Mesh {
    // Layout and GL name
    pub(crate) vao: u32,
    layout: VertexLayout,

    // This specifies some buffers that might've been reassigned externally
    // This hints the mesh that it should try to rebind the attribute's buffer to the VAO
    maybe_reassigned: VertexLayout,

    // Vertex attribute buffers
    pub(super) positions: AttribBuffer<Position>,
    pub(super) normals: AttribBuffer<Normal>,
    pub(super) tangents: AttribBuffer<Tangent>,
    pub(super) colors: AttribBuffer<Color>,
    pub(super) tex_coord_0: AttribBuffer<TexCoord0>,

    // The index buffer (PS: Supports only triangles rn)
    indices: MaybeUninit<ElementBuffer<u32>>,
}



impl Mesh {
    // Create a new mesh from the attribute buffers and the indices
    // The position buffer and index buffer are the only buffers that are required by default
    pub fn from_buffers(
        positions: ArrayBuffer<VePosition>,
        normals: Option<ArrayBuffer<VeNormal>>,
        tangents: Option<ArrayBuffer<VeTangent>>,
        colors: Option<ArrayBuffer<VeColor>>,
        tex_coord_0: Option<ArrayBuffer<VeTexCoord0>>,    
        indices: ElementBuffer<u32>,
    ) -> Option<Self> {
        unsafe {
            let mut mesh = Self { 
                vao: 0,
                layout: VertexLayout::empty(),
                maybe_reassigned: VertexLayout::empty(),
                positions: MaybeUninit::uninit(),
                normals: MaybeUninit::uninit(),
                tangents: MaybeUninit::uninit(),
                colors: MaybeUninit::uninit(),
                tex_coord_0: MaybeUninit::uninit(),
                indices: MaybeUninit::uninit()
            };
            gl::CreateVertexArrays(1, &mut mesh.vao);

            // Set required positions buffer
            mesh.set_attribute::<Position>(Some(positions));

            // Set the optional buffers
            mesh.set_attribute::<Normal>(normals);
            mesh.set_attribute::<Tangent>(tangents);
            mesh.set_attribute::<Color>(colors);
            mesh.set_attribute::<TexCoord0>(tex_coord_0);
            
            // Set required index buffer
            mesh.set_indices(indices);
            mesh.optimize();
            mesh.len().map(|_| mesh)
        }
    } 
    
    // Get the vertex attrib layout that we are using
    pub fn layout(&self) -> VertexLayout {
        self.layout
    }

    // Check if the layout contains a feature
    pub fn contains(&self, feature: VertexLayout) -> bool {
        self.layout.contains(feature)
    }

    // Check if we have a vertex attribute that is enabled and active
    pub fn is_attribute_active<T: Attribute>(&self) -> bool {
        self.contains(T::LAYOUT)
    }

    // Check if this mesh can be used for rendering
    pub fn is_valid(&self) -> bool {
        let tris = self.indices().len() % 3 == 0;
        let required = self.is_attribute_active::<Position>();
        let len = self.len().is_some();
        tris && required && len
    }

    // Get the underlying index buffer immutably
    pub fn indices(&self) -> &ElementBuffer<u32> {
        unsafe { self.indices.assume_init_ref() }
    }

    // Get the underlying index buffer mutably
    pub fn indices_mut(&mut self) -> &mut ElementBuffer<u32> {
        unsafe { self.indices.assume_init_mut() }
    }

    // Set a new element buffer, dropping the old one
    pub fn set_indices(&mut self, buffer: ElementBuffer<u32>) {
        self.indices = MaybeUninit::new(buffer);
    }

    // Get a vertex attribute buffer immutably
    pub fn attribute<T: Attribute>(&self) -> Option<&ArrayBuffer<T::Out>> {
        self.is_attribute_active::<T>().then(|| unsafe { T::assume_init_get(self) })
    }

    // Get a vertex attribute buffer mutably (we can assume that the user assigns a new buffer here)
    pub fn attribute_mut<T: Attribute>(&mut self) -> Option<&mut ArrayBuffer<T::Out>> {
        self.maybe_reassigned.insert(T::LAYOUT);
        self.is_attribute_active::<T>().then(|| unsafe { T::assume_init_get_mut(self) })
    }

    // Set a new vertex attribute buffer, dropping the old one if there was one
    pub fn set_attribute<T: Attribute>(&mut self, buffer: Option<ArrayBuffer<T::Out>>) {
        if let Some(buffer) = buffer {
            // Insert the buffer into the mesh
            unsafe { 
                T::set_raw(self, buffer);
            }
            
            // Enable the vertex attribute and specify it's format
            self.layout.insert(T::LAYOUT);
            self.maybe_reassigned.insert(T::LAYOUT);
            unsafe {
                gl::EnableVertexArrayAttrib(self.vao, T::offset());
                gl::VertexAttribFormat(
                    T::offset(),
                    T::Out::COUNT_PER_VERTEX as i32,
                    T::Out::GL_TYPE,
                    T::NORMALIZED.into(),
                    0
                );
            }
        } else {
            // Disable the vertex attribute
            self.layout.remove(T::LAYOUT);
            unsafe {
                gl::DisableVertexArrayAttrib(self.vao, T::offset())
            }
        }

        // In any case, we must update the 
    }

    // Get the number of vertices that we have in total (this will return None if two or more vectors have different lengths)
    pub fn len(&self) -> Option<usize> {
        let arr = [
            self.attribute::<Position>().map(Buffer::len),
            self.attribute::<Normal>().map(Buffer::len),
            self.attribute::<Tangent>().map(Buffer::len),
            self.attribute::<Color>().map(Buffer::len),
            self.attribute::<TexCoord0>().map(Buffer::len),
        ];

        let first = arr.iter().find(|opt| opt.is_some()).cloned().flatten()?;
        let valid = arr.into_iter().flatten().all(|len| len == first);
        valid.then(|| first)
    }

    // Specify the buffer bindings for all the enabled vertex attributes
    // This will only re-bind the buffer that are marked as "maybe reassigned" since they might be unlinked
    unsafe fn bind_buffers(&mut self) {
        // Bind all the active buffers at the start (create the binding indices)
        let mut buffers = ArrayVec::<u32, 5>::new();
        let mut strides = ArrayVec::<usize, 5>::new();

        if let Some(buffer) = self.attribute::<Position>() {
            buffers.push(buffer.name());
            strides.push(size_of::<VePosition>());
        }

        if let Some(buffer) = self.attribute::<Normal>() {
            buffers.push(buffer.name());
            strides.push(size_of::<VeNormal>());
        }

        if let Some(buffer) = self.attribute::<Tangent>() {
            buffers.push(buffer.name());
            strides.push(size_of::<VeTangent>());
        }

        if let Some(buffer) = self.attribute::<Color>() {
            buffers.push(buffer.name());
            strides.push(size_of::<VeColor>());
        }

        if let Some(buffer) = self.attribute::<TexCoord0>() {
            buffers.push(buffer.name());
            strides.push(size_of::<VeTexCoord0>());
        }
        
        // Bind the vertex buffers
        let offsets = [0; 5];
        //gl::VertexArrayVertexBuffers(self.vao, 0, buffers.len() as i32, buffers.as_ptr(), offsets.as_ptr(), strides.as_ptr());

        // Use the binding indices and link them to the attribute indices for each buffer
        //gl::VertexArrayAttribBinding(self.vao, )
    }

    // Optimize the mesh for rendering (this is called once a frame, for each unique mesh)
    pub fn optimize(&mut self) {
        unsafe {
            self.bind_buffers();
        }
    }

    // Recalculate the vertex normals procedurally; based on position attribute
    // This will fail if the current mesh is not valid
    pub fn compute_normals(&mut self, mode: BufferMode) -> Option<()> {
        self.is_valid().then_some(())?;
        
        // Get positions buffer and mapping
        let mapped = self.attribute::<Position>().unwrap().map();
        let positions = mapped.as_slice();

        // Get index buffer and mapping
        let mapped = self.indices().map();
        let indices = mapped.as_slice();
        
        // Create pre-allocated normal buffer
        let mut normals = vec![vek::Vec3::<f32>::zero(); positions.len()];

        // Normal calculations
        for i in 0..(indices.len() / 3) {
            let i1 = indices[i * 3] as usize;
            let i2 = indices[i * 3 + 1] as usize;
            let i3 = indices[i * 3 + 2] as usize;

            let a = positions[i1];
            let b = positions[i2];
            let c = positions[i3];

            let d1 = b - a;
            let d2 = c - a;
            let cross = vek::Vec3::<f32>::cross(d1, d2).normalized();

            normals[i1] += cross;
            normals[i2] += cross;
            normals[i3] += cross;
        }

        // Normalized + conversion to i8
        let normals: Vec<vek::Vec3<i8>> = normals
            .into_iter()
            .map(|n| 
                n.normalized()
                .map(|e| (e * 127.0) as i8)
            ).collect::<_>();

        self.set_attribute::<Normal>(Some(Buffer::from_slice(normals.as_slice(), mode)));
        Some(())
    }

    // Recalculate the tangents procedurally; based on normal, position, and texture coordinate attributes
    // This will fail if the current mesh is not valid or if the tangent generator fails
    pub fn compute_tangents(&mut self, mode: BufferMode) -> Option<()> {
        self.is_valid().then_some(())?;

        // Get positions slice
        let mapped = self.attribute::<Position>().unwrap().map();
        let positions = mapped.as_slice();

        // Get normals slice
        let mapped = self.attribute::<Normal>().unwrap().map();
        let normals = mapped.as_slice();

        // Get texture coordinate slice
        let mapped = self.attribute::<TexCoord0>().unwrap().map();
        let uvs = mapped.as_slice();

        // Get index slice
        let mapped = self.indices().map();
        let indices = mapped.as_slice();

        // Local struct that will implement the Geometry trait from the tangent generation lib
        struct TangentGenerator<'a> {
            positions: &'a [vek::Vec3<f32>],
            indices: &'a [u32],
            normals: &'a [vek::Vec3<i8>],
            uvs: &'a [vek::Vec2<u8>],
            tangents: &'a mut [vek::Vec4<i8>],
        }

        impl<'a> mikktspace::Geometry for TangentGenerator<'a> {
            fn num_faces(&self) -> usize {
                self.indices.len() / 3
            }
        
            fn num_vertices_of_face(&self, _face: usize) -> usize {
                3
            }
        
            fn position(&self, face: usize, vert: usize) -> [f32; 3] {
                let i = self.indices[face * 3 + vert] as usize;
                self.positions[i].into_array()
            }
        
            fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
                let i = self.indices[face * 3 + vert] as usize;
                self.normals[i].map(|x| x as f32 / 127.0).into_array()
            }
        
            fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
                let i = self.indices[face * 3 + vert] as usize;
                self.uvs[i].map(|x| x as f32 / 255.0).into_array()
            }
        
            fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
                let i = self.indices[face * 3 + vert] as usize;
                self.tangents[i] =
                    vek::Vec4::<f32>::from_slice(&tangent).map(|x| (x * 127.0) as i8);
            }
        }

        let mut tangents = vec![vek::Vec4::<i8>::zero(); positions.len()];
        let mut gen = TangentGenerator {
            positions,
            normals,
            indices,
            uvs,
            tangents: &mut tangents,
        };

        // Generate the procedural tangents and store them
        mikktspace::generate_tangents(&mut gen).then_some(())?;
        self.set_attribute::<Tangent>(Some(Buffer::from_slice(tangents.as_slice(), mode)));
        Some(())
    }

    // Clear the underlying mesh, making it invisible and dispose of the buffers
    pub fn clear(&mut self) {
        let mode = BufferMode::Static;
        *self.indices_mut() = Buffer::empty(mode);
        self.attribute_mut::<Position>().map(|buf| *buf = Buffer::empty(mode));
        self.attribute_mut::<Normal>().map(|buf| *buf = Buffer::empty(mode));
        self.attribute_mut::<Tangent>().map(|buf| *buf = Buffer::empty(mode));
        self.attribute_mut::<Color>().map(|buf| *buf = Buffer::empty(mode));
        self.attribute_mut::<TexCoord0>().map(|buf| *buf = Buffer::empty(mode));
    }

    // Recalculate the AABB bounds of this mesh
    pub fn compute_bounds(&mut self) -> AABB {
        todo!()
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

        

        // Set the very sussy bakas (POV: You are slowly going insane)
        todo!()
    }
}