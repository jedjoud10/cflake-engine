use super::attributes::*;
use crate::mesh::attributes::{Normal, Position, Tangent, TexCoord};
use crate::{
    AttributeBuffer, Direct, Indirect, IndirectMeshArgs, MeshAttribute, MeshAttributes,
    MeshImportError, MeshImportSettings, MeshInitializationError, MultiDrawIndirect,
    MultiDrawIndirectArgs, MultiDrawIndirectCount, MultiDrawIndirectCountArgs, RenderPath,
    TrianglesMut, TrianglesRef, VerticesMut, VerticesRef,
};
use assets::Asset;

use graphics::{
    BufferMode, BufferUsage, DrawCountIndirectBuffer, DrawIndexedIndirectBuffer, Graphics,
    Triangle, TriangleBuffer,
};
use obj::TexturedVertex;

use std::cell::{Cell, RefCell};
use utils::Handle;

// A mesh is a collection of 3D vertices connected by triangles
pub struct Mesh<R: RenderPath = Direct> {
    // Enabled mesh attributes
    enabled: MeshAttributes,

    // Vertex attribute buffers
    positions: Option<R::AttributeBuffer<Position>>,
    normals: Option<R::AttributeBuffer<Normal>>,
    tangents: Option<R::AttributeBuffer<Tangent>>,
    tex_coords: Option<R::AttributeBuffer<TexCoord>>,

    // Custom arguments that are defined by the render path
    args: R::Args,

    // The triangle buffer
    triangles: R::TriangleBuffer<u32>,

    // Cached AABB that we can use for frustum culling
    aabb: Option<math::Aabb<f32>>,
}

impl<R: RenderPath> PartialEq for Mesh<R> {
    fn eq(&self, other: &Self) -> bool {
        self.enabled == other.enabled
            && self.positions == other.positions
            && self.normals == other.normals
            && self.tangents == other.tangents
            && self.tex_coords == other.tex_coords
            && self.args == other.args
            && self.triangles == other.triangles
            && self.aabb == other.aabb
    }
}

pub type IndirectMesh = Mesh<Indirect>;
pub type MultiDrawIndirectMesh = Mesh<MultiDrawIndirect>;
pub type MultiDrawIndirectCountMesh = Mesh<MultiDrawIndirectCount>;

// Initialization of directly rendered meshes
impl Mesh<Direct> {
    // Create a new mesh from the mesh attributes, context, and buffer settings
    pub fn from_slices(
        graphics: &Graphics,
        mode: BufferMode,
        usage: BufferUsage,
        positions: Option<&[RawPosition]>,
        normals: Option<&[RawNormal]>,
        tangents: Option<&[RawTangent]>,
        tex_coords: Option<&[RawTexCoord]>,
        triangles: &[Triangle<u32>],
        aabb: Option<math::Aabb<f32>>,
    ) -> Result<Self, MeshInitializationError> {
        let positions = positions
            .map(|slice| AttributeBuffer::<Position>::from_slice(graphics, slice, mode, usage));
        let normals = normals
            .map(|slice| AttributeBuffer::<Normal>::from_slice(graphics, slice, mode, usage));
        let tangents = tangents
            .map(|slice| AttributeBuffer::<Tangent>::from_slice(graphics, slice, mode, usage));
        let tex_coords = tex_coords
            .map(|slice| AttributeBuffer::<TexCoord>::from_slice(graphics, slice, mode, usage));
        let triangles = Some(TriangleBuffer::from_slice(graphics, triangles, mode, usage));

        let positions = positions
            .transpose()
            .map_err(MeshInitializationError::AttributeBufferInitialization)?;
        let normals = normals
            .transpose()
            .map_err(MeshInitializationError::AttributeBufferInitialization)?;
        let tangents = tangents
            .transpose()
            .map_err(MeshInitializationError::AttributeBufferInitialization)?;
        let tex_coords = tex_coords
            .transpose()
            .map_err(MeshInitializationError::AttributeBufferInitialization)?;
        let triangles = triangles
            .transpose()
            .map_err(MeshInitializationError::TriangleBufferInitialization)?;

        Self::from_buffers(
            positions,
            normals,
            tangents,
            tex_coords,
            triangles.unwrap(),
            aabb,
        )
    }

    // Create a new mesh from the attribute buffers
    pub fn from_buffers(
        positions: Option<AttributeBuffer<Position>>,
        normals: Option<AttributeBuffer<Normal>>,
        tangents: Option<AttributeBuffer<Tangent>>,
        tex_coords: Option<AttributeBuffer<TexCoord>>,
        triangles: TriangleBuffer<u32>,
        aabb: Option<math::Aabb<f32>>,
    ) -> Result<Self, MeshInitializationError> {
        let mut mesh = Self {
            enabled: MeshAttributes::empty(),
            positions: None,
            normals: None,
            tangents: None,
            tex_coords: None,
            args: Some(0),
            triangles,
            aabb,
        };

        // "Set"s a buffer, basically insert it if it's Some and removing it if it's None
        pub fn set<T: MeshAttribute>(
            vertices: &mut VerticesMut<Direct>,
            buffer: Option<AttributeBuffer<T>>,
        ) {
            match buffer {
                Some(x) => vertices.insert::<T>(x),
                None => {
                    vertices.remove::<T>();
                }
            };
        }

        // Set the vertex buffers (including the position buffer)
        let mut vertices = mesh.vertices_mut();
        set::<Position>(&mut vertices, positions);
        set::<Normal>(&mut vertices, normals);
        set::<Tangent>(&mut vertices, tangents);
        set::<TexCoord>(&mut vertices, tex_coords);

        // We don't have to do shit with these since
        // they internally set the data automatically for us
        let _ = vertices.len();

        Ok(mesh)
    }
}

// Initialization of indirectly rendered meshes
impl Mesh<Indirect> {
    // Create a new mesh from the attribute buffers' handles
    pub fn from_handles(
        positions: Option<Handle<AttributeBuffer<Position>>>,
        normals: Option<Handle<AttributeBuffer<Normal>>>,
        tangents: Option<Handle<AttributeBuffer<Tangent>>>,
        tex_coords: Option<Handle<AttributeBuffer<TexCoord>>>,
        triangles: Handle<TriangleBuffer<u32>>,
        indirect: Handle<DrawIndexedIndirectBuffer>,
        offset: usize,
    ) -> Self {
        // Keep track of the enabled mesh buffers
        let mut enabled = MeshAttributes::empty();

        // Inserts the MeshAttribute bitflag of the correspodning attribute if needed
        fn insert<T: MeshAttribute>(
            output: &mut MeshAttributes,
            handle: &Option<Handle<AttributeBuffer<T>>>,
        ) {
            if handle.is_some() {
                output.insert(T::ATTRIBUTE);
            }
        }

        // Update the bitflags
        insert::<Position>(&mut enabled, &positions);
        insert::<Normal>(&mut enabled, &normals);
        insert::<Tangent>(&mut enabled, &tangents);
        insert::<TexCoord>(&mut enabled, &tex_coords);

        // Create the mesh and return it
        Self {
            enabled,
            positions,
            normals,
            tangents,
            tex_coords,
            args: IndirectMeshArgs { indirect, offset },
            triangles,
            aabb: None,
        }
    }

    // Get the indexed indirect buffer handle immutably
    pub fn indirect(&self) -> &Handle<DrawIndexedIndirectBuffer> {
        &self.args.indirect
    }

    // Get the element offset within the DrawIndexedIndirectBuffer
    pub fn offset(&self) -> usize {
        self.args.offset
    }
}

// Initialization of multi-drawn indirect meshes
impl Mesh<MultiDrawIndirect> {
    // Create a new mesh from the attribute buffers' handles
    pub fn from_handles(
        positions: Option<Handle<AttributeBuffer<Position>>>,
        normals: Option<Handle<AttributeBuffer<Normal>>>,
        tangents: Option<Handle<AttributeBuffer<Tangent>>>,
        tex_coords: Option<Handle<AttributeBuffer<TexCoord>>>,
        triangles: Handle<TriangleBuffer<u32>>,
        indirect: Handle<DrawIndexedIndirectBuffer>,
        offset: usize,
        count: usize,
    ) -> Self {
        // Keep track of the enabled mesh buffers
        let mut enabled = MeshAttributes::empty();

        // Inserts the MeshAttribute bitflag of the correspodning attribute if needed
        fn insert<T: MeshAttribute>(
            output: &mut MeshAttributes,
            handle: &Option<Handle<AttributeBuffer<T>>>,
        ) {
            if handle.is_some() {
                output.insert(T::ATTRIBUTE);
            }
        }

        // Update the bitflags
        insert::<Position>(&mut enabled, &positions);
        insert::<Normal>(&mut enabled, &normals);
        insert::<Tangent>(&mut enabled, &tangents);
        insert::<TexCoord>(&mut enabled, &tex_coords);

        // Create the mesh and return it
        Self {
            enabled,
            positions,
            normals,
            tangents,
            tex_coords,
            args: MultiDrawIndirectArgs {
                indirect,
                offset,
                count,
            },
            triangles,
            aabb: None,
        }
    }

    // Get the indexed indirect buffer handle immutably
    pub fn indirect(&self) -> &Handle<DrawIndexedIndirectBuffer> {
        &self.args.indirect
    }

    // Get the element offset within the DrawIndexedIndirectBuffer
    pub fn offset(&self) -> usize {
        self.args.offset
    }

    // Get the element offset within the DrawIndexedIndirectBuffer mutably
    // If offset + count is greater than the number of elements contained within the indexed indirect buffer the program will panic
    pub fn offset_mut(&mut self) -> &mut usize {
        &mut self.args.offset
    }

    // Get the number of draw calls that will be submitted by the GPU
    pub fn count(&self) -> usize {
        self.args.count
    }

    // Get the number od draw calls that will be submitted by the GPU mutably
    // If offset + count is greater than the number of elements contained within the indexed indirect buffer the program will panic
    pub fn count_mut(&mut self) -> &mut usize {
        &mut self.args.count
    }
}

// Initialization of multi-drawn indirect count meshes
impl Mesh<MultiDrawIndirectCount> {
    // Create a new mesh from the attribute buffers' handles
    pub fn from_handles(
        positions: Option<Handle<AttributeBuffer<Position>>>,
        normals: Option<Handle<AttributeBuffer<Normal>>>,
        tangents: Option<Handle<AttributeBuffer<Tangent>>>,
        tex_coords: Option<Handle<AttributeBuffer<TexCoord>>>,
        triangles: Handle<TriangleBuffer<u32>>,
        indirect: Handle<DrawIndexedIndirectBuffer>,
        indirect_offset: usize,
        count: Handle<DrawCountIndirectBuffer>,
        count_offset: usize,
        max_count: usize,
    ) -> Self {
        // Keep track of the enabled mesh buffers
        let mut enabled = MeshAttributes::empty();

        // Inserts the MeshAttribute bitflag of the correspodning attribute if needed
        fn insert<T: MeshAttribute>(
            output: &mut MeshAttributes,
            handle: &Option<Handle<AttributeBuffer<T>>>,
        ) {
            if handle.is_some() {
                output.insert(T::ATTRIBUTE);
            }
        }

        // Update the bitflags
        insert::<Position>(&mut enabled, &positions);
        insert::<Normal>(&mut enabled, &normals);
        insert::<Tangent>(&mut enabled, &tangents);
        insert::<TexCoord>(&mut enabled, &tex_coords);

        // Create the mesh and return it
        Self {
            enabled,
            positions,
            normals,
            tangents,
            tex_coords,
            args: MultiDrawIndirectCountArgs {
                indirect,
                count,
                indirect_offset,
                count_offset,
                max_count,
            },
            triangles,
            aabb: None,
        }
    }

    // Get the indexed indirect buffer handle immutably
    pub fn indirect(&self) -> &Handle<DrawIndexedIndirectBuffer> {
        &self.args.indirect
    }

    // Get the element offset within the DrawIndexedIndirectBuffer
    pub fn indirect_offset(&self) -> usize {
        self.args.indirect_offset
    }

    // Get the element offset within the DrawIndexedIndirectBuffer mutably
    pub fn indirect_offset_mut(&mut self) -> &mut usize {
        &mut self.args.indirect_offset
    }

    // Get the buffer that contains the number of draw calls that will be submitted by the GPU
    pub fn count(&self) -> &Handle<DrawCountIndirectBuffer> {
        &self.args.count
    }

    // Get the element offset within the DrawCountIndirectBuffer
    pub fn count_offset(&self) -> usize {
        self.args.count_offset
    }

    // Get the element offset within the DrawCountIndirectBuffer mutably
    pub fn count_offset_mut(&mut self) -> &mut usize {
        &mut self.args.count_offset
    }

    // Get the maximum number of draw calls that can be submitted immutably
    pub fn max_count(&self) -> usize {
        self.args.max_count
    }

    // Get the maximum number of draw calls that can be submitted mutably
    pub fn max_count_mut(&mut self) -> &mut usize {
        &mut self.args.max_count
    }
}

// Helper functions
impl<R: RenderPath> Mesh<R> {
    // Get a reference to the vertices immutably
    pub fn vertices(&self) -> VerticesRef<'_, R> {
        VerticesRef {
            enabled: self.enabled,
            positions: &self.positions,
            normals: &self.normals,
            tangents: &self.tangents,
            tex_coords: &self.tex_coords,
            count: &self.args,
            aabb: self.aabb,
        }
    }

    // Get a reference to the vertices mutably
    pub fn vertices_mut(&mut self) -> VerticesMut<R> {
        VerticesMut {
            enabled: &mut self.enabled,
            positions: RefCell::new(&mut self.positions),
            normals: RefCell::new(&mut self.normals),
            tangents: RefCell::new(&mut self.tangents),
            tex_coords: RefCell::new(&mut self.tex_coords),
            length_dirty: Cell::new(false),
            aabb_dirty: Cell::new(false),
            count: RefCell::new(&mut self.args),
            aabb: RefCell::new(&mut self.aabb),
        }
    }

    // Get a reference to the triangles immutably
    pub fn triangles(&self) -> TrianglesRef<R> {
        TrianglesRef(&self.triangles)
    }

    // Get a reference to the triangles mutably
    pub fn triangles_mut(&mut self) -> TrianglesMut<R> {
        TrianglesMut(&mut self.triangles)
    }

    // Get the triangles and vertices both at the same time, immutably
    pub fn both(&self) -> (TrianglesRef<R>, VerticesRef<R>) {
        (
            TrianglesRef(&self.triangles),
            VerticesRef {
                enabled: self.enabled,
                positions: &self.positions,
                normals: &self.normals,
                tangents: &self.tangents,
                tex_coords: &self.tex_coords,
                count: &self.args,
                aabb: self.aabb,
            },
        )
    }

    // Get thr triangles and vertices both at the same time, mutably
    pub fn both_mut(&mut self) -> (TrianglesMut<R>, VerticesMut<R>) {
        (
            TrianglesMut(&mut self.triangles),
            VerticesMut {
                enabled: &mut self.enabled,
                positions: RefCell::new(&mut self.positions),
                normals: RefCell::new(&mut self.normals),
                tangents: RefCell::new(&mut self.tangents),
                tex_coords: RefCell::new(&mut self.tex_coords),
                length_dirty: Cell::new(false),
                aabb_dirty: Cell::new(false),
                aabb: RefCell::new(&mut self.aabb),
                count: RefCell::new(&mut self.args),
            },
        )
    }

    // Get the axis-aligned bounding box for this mesh
    // Returns None if the AABB wasn't computed yet or if computation failed
    pub fn aabb(&self) -> Option<math::Aabb<f32>> {
        self.aabb
    }

    // Override the axis aligned bounding box for this mesh
    pub fn set_aabb(&mut self, aabb: Option<math::Aabb<f32>>) {
        self.aabb = aabb;
    }

    // Get the internally stored mesh arguments immutably
    pub fn args(&self) -> &<R as RenderPath>::Args {
        &self.args
    }
}

impl Asset for Mesh {
    type Context<'ctx> = Graphics;
    type Settings<'stg> = MeshImportSettings;
    type Err = MeshImportError;

    fn extensions() -> &'static [&'static str] {
        &["obj"]
    }

    fn deserialize(
        data: assets::Data,
        context: Self::Context<'_>,
        settings: Self::Settings<'_>,
    ) -> Result<Self, Self::Err> {
        let graphics = context;

        // Load the .Obj mesh
        let name = data.path().file_name().unwrap().to_str().unwrap();
        let parsed = obj::load_obj::<TexturedVertex, &[u8], u32>(data.bytes())
            .map_err(MeshImportError::ObjError)?;
        log::debug!(
            "Parsed mesh from file '{}', vertex count: {}, index count: {}",
            name,
            parsed.vertices.len(),
            parsed.indices.len()
        );

        // Create temporary slicetors containing the vertex attributes
        let capacity = parsed.vertices.len();
        let mut positions = Vec::<RawPosition>::with_capacity(capacity);
        let mut normals = settings
            .use_normals
            .then(|| Vec::<RawNormal>::with_capacity(capacity));
        let mut tex_coords = settings
            .use_tex_coords
            .then(|| Vec::<RawTexCoord>::with_capacity(capacity));
        let mut indices = parsed.indices;
        let triangles = bytemuck::cast_slice_mut(&mut indices);
        use vek::{Vec2, Vec3};

        // Convert the vertices into the separate buffer
        for vertex in parsed.vertices {
            // Read and add the position
            positions.push(vek::Vec3::from_slice(&vertex.position).with_w(0f32));

            // Read and add the normal
            if let Some(normals) = &mut normals {
                let read = Vec3::from_slice(&vertex.normal);
                let viewed = read.map(|f| (f * 127.0) as i8);
                normals.push(viewed.with_w(0));
            }

            // Read and add the texture coordinate
            if let Some(tex_coords) = &mut tex_coords {
                let read = Vec2::from_slice(&vertex.texture);
                tex_coords.push(read);
            }
        }

        // Optionally generate the tangents
        let mut tangents = settings.use_tangents.then(|| {
            super::compute_tangents(
                &positions,
                normals.as_ref().unwrap(),
                tex_coords.as_ref().unwrap(),
                triangles,
            )
            .unwrap()
        });

        // Remap the attributes into a slices and options
        let mut normals = normals.as_deref_mut();
        let mut tangents = tangents.as_deref_mut();
        let mut tex_coords = tex_coords.as_deref_mut();

        // Apply the mesh settings to the attributes
        let mut positions = Some(positions.as_mut_slice());
        super::apply_vec_settings(
            settings,
            &mut positions,
            &mut normals,
            &mut tangents,
            &mut tex_coords,
            triangles,
        );

        // Optimize the mesh after we load it
        let triangles = bytemuck::cast_slice_mut(&mut indices);
        super::optimize(
            settings.optimize_vertex_cache,
            settings.optimize_vertex_fetch,
            settings.optimize_overdraw,
            &mut positions,
            &mut normals,
            &mut tangents,
            &mut tex_coords,
            triangles,
        );

        log::debug!(
            "Loaded {} position vertices",
            positions.as_ref().unwrap().len()
        );
        log::debug!(
            "Loaded {} normal vertices",
            normals.as_ref().map(|tc| tc.len()).unwrap_or_default()
        );
        log::debug!(
            "Loaded {} tangent vertices",
            tangents.as_ref().map(|tc| tc.len()).unwrap_or_default()
        );
        log::debug!(
            "Loaded {} texture coordinate vertices",
            tex_coords.as_ref().map(|tc| tc.len()).unwrap_or_default()
        );

        // Create an AABB for this mesh
        let aabb = crate::aabb_from_points(positions.as_ref().unwrap());

        // Generate the mesh and it's corresponding data
        Mesh::from_slices(
            &graphics,
            settings.buffer_mode,
            settings.buffer_usage,
            positions.as_deref(),
            normals.as_deref(),
            tangents.as_deref(),
            tex_coords.as_deref(),
            triangles,
            aabb,
        )
        .map_err(MeshImportError::Initialization)
    }
}
