use super::attributes::*;
use crate::mesh::attributes::{Normal, Position, Tangent, TexCoord};
use crate::{
    AttributeBuffer, MeshAttribute, MeshAttributes, MeshImportError,
    MeshImportSettings, MeshInitializationError, TrianglesMut,
    TrianglesRef, VerticesMut, VerticesRef,
};
use assets::Asset;
use graphics::{
    BufferMode, BufferUsage, Graphics, Triangle, TriangleBuffer, DrawIndexedIndirectBuffer,
};
use obj::TexturedVertex;
use parking_lot::Mutex;
use utils::Handle;
use std::cell::{Cell, RefCell};

// A mesh is a collection of 3D vertices connected by triangles
pub struct Mesh {
    // Enabled mesh attributes
    enabled: MeshAttributes,

    // Vertex attribute buffers
    positions: Option<AttributeBuffer<Position>>,
    normals: Option<AttributeBuffer<Normal>>,
    tangents: Option<AttributeBuffer<Tangent>>,
    tex_coords: Option<AttributeBuffer<TexCoord>>,

    // The number of vertices stored in this mesh
    // None if the buffers contain different sizes
    len: Option<usize>,

    // The triangle buffer
    triangles: TriangleBuffer<u32>,

    // Cached AABB that we can use for frustum culling
    aabb: Option<math::Aabb<f32>>,
}

// Mesh initialization for 3D meshes
impl Mesh {
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
    ) -> Result<Self, MeshInitializationError> {
        let positions = positions.map(|slice| {
            AttributeBuffer::<Position>::from_slice(
                graphics, slice, mode, usage,
            )
            .unwrap()
        });
        let normals = normals.map(|slice| {
            AttributeBuffer::<Normal>::from_slice(
                graphics, slice, mode, usage,
            )
            .unwrap()
        });
        let tangents = tangents.map(|slice| {
            AttributeBuffer::<Tangent>::from_slice(
                graphics, slice, mode, usage,
            )
            .unwrap()
        });
        let tex_coords = tex_coords.map(|slice| {
            AttributeBuffer::<TexCoord>::from_slice(
                graphics, slice, mode, usage,
            )
            .unwrap()
        });
        let triangles = TriangleBuffer::from_slice(
            graphics, triangles, mode, usage,
        )
        .unwrap();
        Self::from_buffers(
            positions, normals, tangents, tex_coords, triangles,
        )
    }

    // Create a new mesh from the attribute buffers
    pub fn from_buffers(
        positions: Option<AttributeBuffer<Position>>,
        normals: Option<AttributeBuffer<Normal>>,
        tangents: Option<AttributeBuffer<Tangent>>,
        tex_coords: Option<AttributeBuffer<TexCoord>>,
        triangles: TriangleBuffer<u32>,
    ) -> Result<Self, MeshInitializationError> {
        let mut mesh = Self {
            enabled: MeshAttributes::empty(),
            positions: None,
            normals: None,
            tangents: None,
            tex_coords: None,
            len: Some(0),
            triangles,
            aabb: None,
        };

        // "Set"s a buffer, basically insert it if it's Some and removing it if it's None
        pub fn set<T: MeshAttribute>(
            vertices: &mut VerticesMut,
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
        let _ = vertices.aabb();

        Ok(mesh)
    }
}

// Helper functions
impl Mesh {
    // Get a reference to the vertices immutably
    pub fn vertices<'a>(&'a self) -> VerticesRef<'a> {
        VerticesRef {
            enabled: self.enabled,
            positions: &self.positions,
            normals: &self.normals,
            tangents: &self.tangents,
            tex_coords: &self.tex_coords,
            len: self.len,
            aabb: self.aabb,
        }
    }

    // Get a reference to the vertices mutably
    pub fn vertices_mut(&mut self) -> VerticesMut {
        VerticesMut {
            enabled: &mut self.enabled,
            positions: RefCell::new(&mut self.positions),
            normals: RefCell::new(&mut self.normals),
            tangents: RefCell::new(&mut self.tangents),
            tex_coords: RefCell::new(&mut self.tex_coords),
            length_dirty: Cell::new(false),
            aabb_dirty: Cell::new(false),
            len: RefCell::new(&mut self.len),
            aabb: RefCell::new(&mut self.aabb),
        }
    }

    // Get a reference to the triangles immutably
    pub fn triangles(&self) -> TrianglesRef {
        TrianglesRef(&self.triangles)
    }

    // Get a reference to the triangles mutably
    pub fn triangles_mut(&mut self) -> TrianglesMut {
        TrianglesMut(&mut self.triangles)
    }

    // Get the triangles and vertices both at the same time, immutably
    pub fn both(&self) -> (TrianglesRef, VerticesRef) {
        (
            TrianglesRef(&self.triangles),
            VerticesRef {
                enabled: self.enabled,
                positions: &self.positions,
                normals: &self.normals,
                tangents: &self.tangents,
                tex_coords: &self.tex_coords,
                len: self.len,
                aabb: self.aabb,
            },
        )
    }

    // Get thr triangles and vertices both at the same time, mutably
    pub fn both_mut(&mut self) -> (TrianglesMut, VerticesMut) {
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
                len: RefCell::new(&mut self.len),
                aabb: RefCell::new(&mut self.aabb),
            },
        )
    }

    // Get the axis-aligned bounding box for this mesh
    // Returns None if the AABB wasn't computed yet or if computation failed
    pub fn aabb(&mut self) -> Option<math::Aabb<f32>> {
        self.aabb
    }
}

// An indirect mesh contains a handle to multiple buffers. It doesn't own them directly
// Sole reason I did this is because we can set the vertex buffers only one, and use the
// offset within DrawIndexedIndirect to offset it for each mesh. A lot more memory can be saved
pub struct IndirectMesh {
    // Enabled mesh attributes
    enabled: MeshAttributes,

    // Vertex attribute buffers
    positions: Option<Handle<AttributeBuffer<Position>>>,
    normals: Option<Handle<AttributeBuffer<Normal>>>,
    tangents: Option<Handle<AttributeBuffer<Tangent>>>,
    tex_coords: Option<Handle<AttributeBuffer<TexCoord>>>,

    // The triangle buffer
    triangles: Handle<TriangleBuffer<u32>>,

    // Indirect draw buffer
    indirect: Handle<DrawIndexedIndirectBuffer>,

    // Manual AABB that we can use for frustum culling
    aabb: Option<math::Aabb<f32>>,
}

// Mesh initialization for 3D meshes
impl IndirectMesh {
    // Create a new mesh from the attribute buffers' handles
    pub fn from_handles(
        positions: Option<Handle<AttributeBuffer<Position>>>,
        normals: Option<Handle<AttributeBuffer<Normal>>>,
        tangents: Option<Handle<AttributeBuffer<Tangent>>>,
        tex_coords: Option<Handle<AttributeBuffer<TexCoord>>>,
        triangles: Handle<TriangleBuffer<u32>>,
        indirect: Handle<DrawIndexedIndirectBuffer>,
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
            aabb: None,
            indirect,
            triangles,
            enabled,
            positions,
            normals,
            tangents,
            tex_coords,
        }
    }
}

// Helper functions
impl IndirectMesh {
}

impl Asset for Mesh {
    type Context<'ctx> = Graphics;
    type Settings<'stg> = MeshImportSettings;
    type Err = MeshImportError;

    fn extensions() -> &'static [&'static str] {
        &["obj"]
    }

    fn deserialize<'c, 's>(
        data: assets::Data,
        context: Self::Context<'c>,
        settings: Self::Settings<'s>,
    ) -> Result<Self, Self::Err> {
        let graphics = context;

        // Load the .Obj mesh
        let name = data.path().file_name().unwrap().to_str().unwrap();
        let parsed =
            obj::load_obj::<TexturedVertex, &[u8], u32>(data.bytes())
                .map_err(MeshImportError::ObjError)?;
        log::debug!("Parsed mesh from file '{}', vertex count: {}, index count: {}", name, parsed.vertices.len(), parsed.indices.len());

        // Create temporary slicetors containing the vertex attributes
        let capacity = parsed.vertices.len();
        let mut positions =
            Vec::<RawPosition>::with_capacity(capacity);
        let mut normals = settings
            .use_normals
            .then(|| Vec::<RawNormal>::with_capacity(capacity));
        let mut tex_coords = settings
            .use_tex_coords
            .then(|| Vec::<RawTexCoord>::with_capacity(capacity));
        let mut triangles =
            Vec::<[u32; 3]>::with_capacity(parsed.indices.len() / 3);
        let indices = parsed.indices;
        use vek::{Vec2, Vec3};

        // Convert the vertices into the separate buffer
        for vertex in parsed.vertices {
            // Read and add the position
            positions.push(
                vek::Vec3::from_slice(&vertex.position).with_w(0f32),
            );

            // Read and add the normal
            if let Some(normals) = &mut normals {
                let read = Vec3::from_slice(&vertex.normal);
                let viewed = read.map(|f| (f * 127.0) as i8);
                normals.push(viewed.with_w(0));
            }

            // Read and add the texture coordinate
            if let Some(tex_coords) = &mut tex_coords {
                let read = Vec2::from_slice(&vertex.texture);
                let viewed = read.map(|f| (f * 255.0) as u8);
                tex_coords.push(viewed.with_w(0));
            }
        }

        // Convert the indices to triangles
        for triangle in indices.chunks_exact(3) {
            triangles.push(triangle.try_into().unwrap());
        }

        // Optionally generate the tangents
        let mut tangents = settings.use_tangents.then(|| {
            super::compute_tangents(
                &positions,
                normals.as_ref().unwrap(),
                tex_coords.as_ref().unwrap(),
                &triangles,
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
            &mut triangles,
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
            tex_coords
                .as_ref()
                .map(|tc| tc.len())
                .unwrap_or_default()
        );

        // Generate the mesh and it's corresponding data
        Mesh::from_slices(
            &graphics,
            settings.buffer_mode,
            settings.buffer_usage,
            positions.as_deref(),
            normals.as_deref(),
            tangents.as_deref(),
            tex_coords.as_deref(),
            &triangles,
        )
        .map_err(MeshImportError::Initialization)
    }
}
