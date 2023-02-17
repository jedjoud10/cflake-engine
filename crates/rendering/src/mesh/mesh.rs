use assets::Asset;
use graphics::{TriangleBuffer, Graphics, BufferMode, BufferUsage, Triangle, VertexBuffer, Vertex};
use obj::TexturedVertex;
use super::attributes::*;
use crate::{AttributeBuffer, EnabledMeshAttributes, MeshImportSettings, MeshImportError, MeshInitializationError, MeshAttribute, VerticesRef, VerticesMut, TrianglesRef, TrianglesMut};
use crate::mesh::attributes::{TexCoord, Tangent, Normal, Position};
use std::cell::{Cell, RefCell};
use std::mem::MaybeUninit;

// A mesh is a collection of 3D vertices connected by triangles
pub struct Mesh {
    // Enabled mesh attributes
    enabled: EnabledMeshAttributes,

    // Vertex attribute buffers
    positions: MaybeUninit<AttributeBuffer<Position>>,
    normals: MaybeUninit<AttributeBuffer<Normal>>,
    tangents: MaybeUninit<AttributeBuffer<Tangent>>,
    tex_coords: MaybeUninit<AttributeBuffer<TexCoord>>,

    // The number of vertices stored in this mesh
    // None if the buffers contain different sizes
    len: Option<usize>,

    // The triangle buffer
    triangles: TriangleBuffer<u32>,
}

// Mesh initialization for 3D meshes
impl Mesh {
    // Create a new mesh from the mesh attributes, context, and buffer settings
    // TODO: Support multiple modes and usages PER attribute
    
    // FIXME: Crashes when trying to create triangles buffer for the default engine sphere
    // Has to do with the high triangle, maybe usize overflow?
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
        let positions = positions.map(|slice| AttributeBuffer::<Position>::from_slice(graphics, &slice, mode, usage).unwrap());
        let normals = normals.map(|slice| AttributeBuffer::<Normal>::from_slice(graphics, &slice, mode, usage).unwrap());
        let tangents = tangents.map(|slice| AttributeBuffer::<Tangent>::from_slice(graphics, &slice, mode, usage).unwrap());
        let tex_coords = tex_coords.map(|slice| AttributeBuffer::<TexCoord>::from_slice(graphics, &slice, mode, usage).unwrap());
        let triangles = TriangleBuffer::from_slice(graphics, &triangles, mode, usage).unwrap();
        Self::from_buffers(positions, normals, tangents, tex_coords, triangles)
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
            enabled: EnabledMeshAttributes::empty(),
            positions: MaybeUninit::uninit(),
            normals: MaybeUninit::uninit(),
            tangents: MaybeUninit::uninit(),
            tex_coords: MaybeUninit::uninit(),
            len: Some(0),
            triangles,
        };

        // "Set"s a buffer, basically insert it if it's Some and removing it if it's None
        pub fn set<T: MeshAttribute>(vertices: &mut VerticesMut, buffer: Option<AttributeBuffer<T>>) {
            match buffer {
                Some(x) => vertices.insert::<T>(x),
                None => { vertices.remove::<T>(); },
            };
        } 

        // Set the vertex buffers (including the position buffer)
        let mut vertices = mesh.vertices_mut();
        set::<Position>(&mut vertices, positions);
        set::<Normal>(&mut vertices, normals);
        set::<Tangent>(&mut vertices, tangents);
        set::<TexCoord>(&mut vertices, tex_coords);
        let len = vertices.len();
        mesh.len = len;
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
        }
    }

    // Get a reference to the vertices mutably
    pub fn vertices_mut(&mut self) -> VerticesMut {
        VerticesMut {
            enabled: &mut self.enabled,
            positions: &mut self.positions,
            normals: &mut self.normals,
            tangents: &mut self.tangents,
            tex_coords: &mut self.tex_coords,
            len: RefCell::new(&mut self.len),
            dirty: Cell::new(false),
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
            },
        )
    }

    // Get thr triangles and vertices both at the same time, mutably
    pub fn both_mut(&mut self) -> (TrianglesMut, VerticesMut) {
        (
            TrianglesMut(&mut self.triangles),
            VerticesMut {
                enabled: &mut self.enabled,
                positions: &mut self.positions,
                normals: &mut self.normals,
                tangents: &mut self.tangents,
                tex_coords: &mut self.tex_coords,
                len: RefCell::new(&mut self.len),
                dirty: Cell::new(false),
            },
        )
    }
}

impl Asset for Mesh {
    type Context<'ctx> = &'ctx Graphics;
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
        let parsed = obj::load_obj::<TexturedVertex, &[u8], u32>(data.bytes())
            .map_err(MeshImportError::ObjError)?;
        log::debug!("Parsed mesh from file '{}', vertex count: {}, index count: {}", name, parsed.vertices.len(), parsed.indices.len());

        // Create temporary slicetors containing the vertex attributes
        let capacity = parsed.vertices.len();
        let mut positions = Vec::<RawPosition>::with_capacity(capacity);
        let mut normals = settings
            .use_normals
            .then(|| Vec::<RawNormal>::with_capacity(capacity));
        let mut tex_coords = settings
            .use_tex_coords
            .then(|| Vec::<RawTexCoord>::with_capacity(capacity));
        let mut triangles = Vec::<[u32; 3]>::with_capacity(parsed.indices.len() / 3);
        let indices = parsed.indices;
        use vek::{Vec2, Vec3};

        // Convert the vertices into the separate buffer
        for vertex in parsed.vertices {
            // Read and add the position
            positions.push(vek::Vec3::from_slice(&vertex.position));

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
                tex_coords.push(viewed);
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
        let mut normals = normals.as_mut().map(|vec| vec.as_mut_slice());
        let mut tangents = tangents.as_mut().map(|vec| vec.as_mut_slice());
        let mut tex_coords = tex_coords.as_mut().map(|vec| vec.as_mut_slice());

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

        log::debug!("Loaded {} position vertices", positions.as_ref().unwrap().len());
        log::debug!("Loaded {} normal vertices", normals.as_ref().map(|tc| tc.len()).unwrap_or_default());
        log::debug!("Loaded {} tangent vertices", tangents.as_ref().map(|tc| tc.len()).unwrap_or_default());
        log::debug!("Loaded {} texture coordinate vertices", tex_coords.as_ref().map(|tc| tc.len()).unwrap_or_default());


        // Generate the mesh and it's corresponding data
        Mesh::from_slices(
            graphics,
            settings.buffer_mode,
            settings.buffer_usage,
            positions.as_deref(),
            normals.as_deref(),
            tangents.as_deref(),
            tex_coords.as_deref(),
            &triangles
        ).map_err(MeshImportError::Initialization)
    }
}