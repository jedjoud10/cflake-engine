use std::{mem::{MaybeUninit, size_of}, ptr::null};

use arrayvec::ArrayVec;
use assets::Asset;
use math::AABB;
use obj::TexturedVertex;
use rayon::iter::Positions;

use super::{attributes::*, MeshImportSettings};
use crate::{
    buffer::{Buffer, ElementBuffer, ArrayBuffer, BufferMode, BufferAnyRef},
    context::Context, mesh::{attributes::RawAttribute, MeshImportMode}, prelude::Array, object::{ToGlName, Shared},
};

// Contains the underlying array buffer for a specific attribute
type AttribBuffer<A> = MaybeUninit<ArrayBuffer<<A as Attribute>::Out>>;

// A mesh is a collection of 3D vertices connected by triangles
// Each sub-mesh is associated with a single material
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
    pub(super) tex_coord: AttribBuffer<TexCoord>,

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
        tex_coord: Option<ArrayBuffer<VeTexCoord0>>,    
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
                tex_coord: MaybeUninit::uninit(),
                indices: MaybeUninit::uninit()
            };
            gl::CreateVertexArrays(1, &mut mesh.vao);

            // Set required positions buffer
            mesh.set_attribute::<Position>(Some(positions));

            // Set the optional buffers
            mesh.set_attribute::<Normal>(normals);
            mesh.set_attribute::<Tangent>(tangents);
            mesh.set_attribute::<Color>(colors);
            mesh.set_attribute::<TexCoord>(tex_coord);
            
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

    // Get an array containing all the buffer any ref and attribute format any
    pub fn attributes_any(&self) -> [Option<(BufferAnyRef, AttributeFormatAny)>; MAX_MESH_VERTEX_ATTRIBUTES] {
        [
            self.attribute::<Position>().map(|b| (Buffer::as_buffer_any_ref(b), Position::as_attribute_any())),
            self.attribute::<Normal>().map(|b| (Buffer::as_buffer_any_ref(b), Normal::as_attribute_any())),
            self.attribute::<Tangent>().map(|b| (Buffer::as_buffer_any_ref(b), Tangent::as_attribute_any())),
            self.attribute::<Color>().map(|b| (Buffer::as_buffer_any_ref(b), Color::as_attribute_any())),
            self.attribute::<TexCoord>().map(|b| (Buffer::as_buffer_any_ref(b), TexCoord::as_attribute_any())),
        ]
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
                gl::EnableVertexArrayAttrib(self.vao, T::attribute_index());
                gl::VertexAttribFormat(
                    T::attribute_index(),
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
                gl::DisableVertexArrayAttrib(self.vao, T::attribute_index())
            }
        }
    }

    // Get the number of vertices that we have in total (this will return None if two or more vectors have different lengths)
    pub fn len(&self) -> Option<usize> {
        let mut arr = self
            .attributes_any()
            .into_iter()
            .map(|opt| 
                opt.map(|(buf, _)| buf.len()
            )
        );

        let first = arr.find(|opt| opt.is_some()).flatten()?;
        let valid = arr.into_iter().flatten().all(|len| len == first);
        valid.then(|| first)
    }

    // Specify the buffer bindings for all the enabled vertex attributes
    // This will only re-bind the buffer that are marked as "maybe reassigned" since they might be unlinked
    unsafe fn bind_buffers(&mut self) {
        // Bind all the active buffers at the start (create the binding indices)
        let iter = self.attributes_any().into_iter().filter_map(|s| s).enumerate();
        for (i, (buffer, attrib)) in iter {
            if self.maybe_reassigned.contains(attrib.layout()) {
                gl::VertexArrayVertexBuffer(self.vao, i as u32, buffer.name(), 0, buffer.stride() as i32);
                gl::VertexArrayAttribBinding(self.vao, attrib.attribute_index(), i as u32)
            }
        }
    }

    // Optimize the mesh for rendering (this is called once a frame, for each unique mesh)
    pub fn optimize(&mut self) {
        unsafe {
            self.bind_buffers();
        }
    }

    // Recalculate the vertex normals procedurally; based on position attribute
    // This will fail if the current mesh is not valid
    pub fn compute_normals(&mut self, ctx: &mut Context, mode: BufferMode) -> Option<()> {
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

        self.set_attribute::<Normal>(Some(Buffer::from_slice(ctx, normals.as_slice(), mode)));
        Some(())
    }

    // Recalculate the tangents procedurally; based on normal, position, and texture coordinate attributes
    // This will fail if the current mesh is not valid or if the tangent generator fails
    pub fn compute_tangents(&mut self, ctx: &mut Context, mode: BufferMode) -> Option<()> {
        self.is_valid().then_some(())?;

        // Get positions slice
        let mapped = self.attribute::<Position>().unwrap().map();
        let positions = mapped.as_slice();

        // Get normals slice
        let mapped = self.attribute::<Normal>().unwrap().map();
        let normals = mapped.as_slice();

        // Get texture coordinate slice
        let mapped = self.attribute::<TexCoord>().unwrap().map();
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
        self.set_attribute::<Tangent>(Some(Buffer::from_slice(ctx, tangents.as_slice(), mode)));
        Some(())
    }

    // Clear the underlying mesh, making it invisible and dispose of the buffers
    pub fn clear(&mut self, ctx: &mut Context) {
        let mode = BufferMode::Static;
        *self.indices_mut() = Buffer::empty(ctx, mode);
        self.attribute_mut::<Position>().map(|buf| *buf = Buffer::empty(ctx, mode));
        self.attribute_mut::<Normal>().map(|buf| *buf = Buffer::empty(ctx, mode));
        self.attribute_mut::<Tangent>().map(|buf| *buf = Buffer::empty(ctx, mode));
        self.attribute_mut::<Color>().map(|buf| *buf = Buffer::empty(ctx, mode));
        self.attribute_mut::<TexCoord>().map(|buf| *buf = Buffer::empty(ctx, mode));
    }

    // Recalculate the AABB bounds of this mesh
    pub fn compute_bounds(&mut self) -> AABB {
        todo!()
    }
}
impl<'a> Asset<'a> for Mesh {
    type Args = (&'a mut Context, MeshImportSettings);

    fn extensions() -> &'static [&'static str] {
        &["obj"]
    }

    fn deserialize(data: assets::Data, args: Self::Args) -> Self {
        let (ctx, settings) = args;

        // Load the .Obj mesh
        let parsed = obj::load_obj::<TexturedVertex, &[u8], u32>(data.bytes()).unwrap();
        
        // Create temporary vectors containing the vertex attributes
        let capacity = parsed.vertices.len();
        let mut positions = Vec::with_capacity(capacity);
        let mut normals = Vec::with_capacity(capacity);
        let mut tex_coords_0 = Vec::with_capacity(capacity);
        let indices = parsed.indices;

        use vek::{Vec2, Vec3};

        // Convert the vertices into the separate buffer
        for vertex in parsed.vertices {
            positions.push(Vec3::from_slice(&vertex.position));
            normals.push(Vec3::from_slice(&vertex.normal).map(|f| (f * 127.0) as i8));
            tex_coords_0.push(Vec2::from_slice(&vertex.texture).map(|f| (f * 255.0) as u8));
        }        

        // Convert the mesh mode into the valid buffer modes
        let mode = match settings.mode {
            MeshImportMode::Static => BufferMode::Static,
            MeshImportMode::Dynamic => BufferMode::Dynamic,
            MeshImportMode::Procedural => BufferMode::Resizable,
        };

        // Create the buffers
        let positions = Buffer::from_slice(ctx, &positions, mode);
        let normals = Buffer::from_slice(ctx, &normals, mode);
        let tex_coord = Buffer::from_slice(ctx, &tex_coords_0, mode);
        let indices = Buffer::from_slice(ctx, &indices, mode);

        // Create a new mesh
        let mut mesh = Mesh::from_buffers(positions, Some(normals), None, None, Some(tex_coord), indices).unwrap();

        // Generate procedural tangents if requested
        if settings.generate_tangents {
            mesh.compute_tangents(ctx, mode).unwrap();
        }
        mesh
    }
}