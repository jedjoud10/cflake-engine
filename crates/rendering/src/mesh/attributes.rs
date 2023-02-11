use std::mem::MaybeUninit;
use arrayvec::ArrayVec;
use graphics::{VertexBuffer, Vertex, XYZ, XYZW, XY, Normalized, VertexConfig, GpuPodRelaxed, VertexInput, PerVertex, VertexInfo, VertexInputInfo};
use std::marker::PhantomData;
use paste::paste;

use crate::{VerticesMut, VerticesRef};

bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct EnabledMeshAttributes: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 1;
        const TANGENTS = 1 << 2;
        //const COLORS = 1 << 3;
        const TEX_COORDS = 1 << 4;
    }
}

// This is the maximum number of active attributes that we can have inside a mesh
pub const MAX_MESH_VERTEX_ATTRIBUTES: usize =
    EnabledMeshAttributes::all().bits.count_ones() as usize;

// Contains the underlying array buffer for a specific attribute
pub type AttributeBuffer<A> = MaybeUninit<VertexBuffer<<<A as MeshAttribute>::V as Vertex>::Storage>>;

// A named attribute that has a specific name, like "Position", or "Normal"
pub trait MeshAttribute {
    type V: Vertex;
    type Storage: GpuPodRelaxed;
    type Input: VertexInput<Self::V>;
    const ATTRIBUTE: EnabledMeshAttributes;

    // Try to get the references to the underlying vertex buffers
    // Forgive me my children, for I have failed to bring you salvation, from this cold, dark, world...
    // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
    fn from_ref_as_ref<'a>(vertices: &'a VerticesRef) -> Option<&'a VertexBuffer<Self::Storage>>;
    fn from_mut_as_mut<'a>(vertices: &'a mut VerticesMut) -> Option<&'a mut VertexBuffer<Self::Storage>>;
    fn from_mut_as_ref<'a>(vertices: &'a VerticesMut) -> Option<&'a VertexBuffer<Self::Storage>>;

    // Insert a mesh attribute vertex buffer into the vertices
    fn insert(vertices: &mut VerticesMut, buffer: VertexBuffer<Self::Storage>);

    // Try to remove the mesh attribute vertex buffer from the vertices
    fn remove(vertices: &mut VerticesMut) -> Option<VertexBuffer<Self::Storage>>;
    
    // Get the attribute's index
    fn index() -> u32 {
        debug_assert_eq!(Self::ATTRIBUTE.bits().count_ones(), 1);
        Self::ATTRIBUTE.bits().trailing_zeros()
    }
}

// Get a list of the untyped attributes from the enabled mesh attributes
pub(crate) fn enabled_to_vertex_config(attributes: EnabledMeshAttributes) -> VertexConfig {
    // This will push the mesh attribute's input to the vector if the bitflags contain the vertex input
    fn push<M: MeshAttribute>(attributes: EnabledMeshAttributes, inputs: &mut Vec<VertexInputInfo>) {
        if attributes.contains(M::ATTRIBUTE) {
            let input = <M::Input as VertexInput<M::V>>::new(M::index());
            inputs.push(input.info());
        }
    }
    
    // Add the different types of mesh attributes
    let mut inputs = Vec::<VertexInputInfo>::new();
    push::<Position>(attributes, &mut inputs);
    push::<Normal>(attributes, &mut inputs);
    push::<Tangent>(attributes, &mut inputs);
    push::<TexCoord>(attributes, &mut inputs);
    
    VertexConfig {
        inputs
    }
}

macro_rules! impl_vertex_attribute {
    ($attribute:ident, $name:ident, $vertex:ty, $enabled:ident, $input:ident) => {
        paste! {
            pub struct $attribute(PhantomData<$vertex>);
            pub type [<Raw $attribute>] = <<$attribute as MeshAttribute>::V as Vertex>::Storage;
            
            impl MeshAttribute for $attribute {
                type V = $vertex;
                type Storage = <$vertex as Vertex>::Storage;
                type Input = $input<Self::V>;
                const ATTRIBUTE: EnabledMeshAttributes = EnabledMeshAttributes::[<$enabled>];

                fn from_ref_as_ref<'a>(vertices: &'a VerticesRef) -> Option<&'a VertexBuffer<Self::Storage>> {
                    vertices.is_enabled::<Self>().then(|| {
                        unsafe { vertices.$name.assume_init_ref() }
                    })
                }

                fn from_mut_as_mut<'a>(vertices: &'a mut VerticesMut) -> Option<&'a mut VertexBuffer<Self::Storage>> {
                    vertices.is_enabled::<Self>().then(|| {
                        unsafe { vertices.$name.assume_init_mut() }
                    })
                }

                fn from_mut_as_ref<'a>(vertices: &'a VerticesMut) -> Option<&'a VertexBuffer<Self::Storage>> {
                    vertices.is_enabled::<Self>().then(|| {
                        unsafe { vertices.$name.assume_init_ref() }
                    })
                }
            
                fn insert(vertices: &mut VerticesMut, buffer: VertexBuffer<Self::Storage>) {
                    if vertices.is_enabled::<Self>() {
                        let mut old = std::mem::replace(vertices.$name, std::mem::MaybeUninit::new(buffer));
                        unsafe { old.assume_init_drop() }
                    } else {
                        *vertices.$name = std::mem::MaybeUninit::new(buffer);
                    }

                    vertices.enabled.insert(Self::ATTRIBUTE);
                }
            
                fn remove<'a>(vertices: &mut VerticesMut<'a>) -> Option<VertexBuffer<Self::Storage>> {
                    vertices.enabled.remove(Self::ATTRIBUTE);
                    vertices.is_enabled::<Self>().then(|| {
                        std::mem::replace(vertices.$name, std::mem::MaybeUninit::uninit())
                    }).map(|x| unsafe { x.assume_init() })
                }
            }
        }
    };
}

impl_vertex_attribute!(Position, positions, XYZ<f32>, POSITIONS, PerVertex);
impl_vertex_attribute!(Normal, normals, XYZW<Normalized<i8>>, NORMALS, PerVertex);
impl_vertex_attribute!(Tangent, tangents, XYZW<Normalized<i8>>, TANGENTS, PerVertex);
//impl_vertex_attribute!(Color, colors, XYZ<Normalized<u8>>, COLORS);
impl_vertex_attribute!(TexCoord, tex_coords, XY<Normalized<u8>>, TEX_COORDS, PerVertex);