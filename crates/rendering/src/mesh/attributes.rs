use std::mem::MaybeUninit;
use arrayvec::ArrayVec;
use graphics::{VertexBuffer, Vertex, VertexBinding, VertexAttribute, UntypedVertex, XYZ, XYZW, XY, Normalized, VertexConfig};
use std::marker::PhantomData;
use paste::paste;

#[cfg(not(feature = "two-dim"))]
bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct EnabledMeshAttributes: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 1;
        const TANGENTS = 1 << 2;
        const COLORS = 1 << 3;
        const TEX_COORDS = 1 << 4;
    }
}

#[cfg(feature = "two-dim")]
bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct EnabledMeshAttributes: u8 {
        const POSITIONS = 1;
        const COLORS = 1 << 3;
    }
}

// This is the maximum number of active attributes that we can have inside a mesh
pub const MAX_MESH_VERTEX_ATTRIBUTES: usize =
    EnabledMeshAttributes::all().bits.trailing_ones() as usize;

// Contains the underlying array buffer for a specific attribute
pub type AttributeBuffer<A> = MaybeUninit<VertexBuffer<<<A as MeshAttribute>::V as Vertex>::Storage>>;

// An untyped attribute wrapper that contains all the basic information about attributes
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UntypedMeshAttribute {
    pub untyped: UntypedVertex,
    pub enabled: EnabledMeshAttributes,
    pub attribute: VertexAttribute,
    pub binding: VertexBinding,
}

// A named attribute that has a specific name, like "Position", or "Normal"
pub trait MeshAttribute {
    type V: Vertex;
    const ATTRIBUTE: EnabledMeshAttributes;

    /*
    // Get the proper reference from the wrapper vertex types
    fn from_vertices_ref_as_ref<'a>(vertices: &'a VerticesRef) -> &'a AttributeBuffer<Self>;
    fn from_vertices_mut_as_ref<'a>(vertices: &'a VerticesMut) -> &'a AttributeBuffer<Self>;
    fn from_vertices_mut_as_mut<'a>(vertices: &'a mut VerticesMut) -> &'a mut AttributeBuffer<Self>;

    // Insert an attribute buffer into the vertices
    fn insert(vertices: &mut VerticesMut, buffer: ArrayBuffer<Self::Out>);

    // Remove an attribute from the vertices
    fn remove(vertices: &mut VerticesMut);
    */

    // Get the attribute's index
    fn index() -> u32 {
        debug_assert_eq!(Self::ATTRIBUTE.bits().count_ones(), 1);
        Self::ATTRIBUTE.bits().trailing_zeros()
    }

    // Needed for the graphics pipeline' VertexConfig
    fn binding() -> VertexBinding;
    fn attribute() -> VertexAttribute;

    // Get the attribute's format as an untyped struct
    fn untyped() -> UntypedMeshAttribute {
        UntypedMeshAttribute { 
            untyped: <Self::V as Vertex>::untyped(),
            enabled: Self::ATTRIBUTE,
            attribute: Self::attribute(),
            binding: Self::binding(),
        }
    }
}

// Get a list of the untyped attributes from the enabled mesh attributes
pub fn untyped_attributes_from_enabled_attributes(attributes: EnabledMeshAttributes) -> ArrayVec<UntypedMeshAttribute, MAX_MESH_VERTEX_ATTRIBUTES> {
    let mut vec = ArrayVec::new();

    // Add the attribute's untyped representation to the vector if it's enabled
    fn push<A: MeshAttribute>(attributes: EnabledMeshAttributes, vec: &mut ArrayVec<UntypedMeshAttribute, MAX_MESH_VERTEX_ATTRIBUTES>) {
        if attributes.contains(Position::ATTRIBUTE) {
            vec.push(Position::untyped())
        }
    }

    // Add the mesh attributes untyped representations
    push::<Position>(attributes, &mut vec);
    push::<Normal>(attributes, &mut vec);
    push::<Tangent>(attributes, &mut vec);
    push::<Color>(attributes, &mut vec);
    push::<TexCoord>(attributes, &mut vec);
    vec
}


macro_rules! impl_vertex_attribute {
    ($attribute:ident, $name:ident, $vertex:ty, $enabled:ident) => {
        paste! {
            pub struct $attribute(PhantomData<$vertex>);
            pub type [<Ve $attribute>] = <<$attribute as MeshAttribute>::V as Vertex>::Storage;
            
            impl MeshAttribute for $attribute {
                type V = $vertex;
                const ATTRIBUTE: EnabledMeshAttributes = EnabledMeshAttributes::[<$enabled>];

                fn binding() -> VertexBinding {
                    VertexBinding {
                        binding: Self::index(),
                        format: <Self::V as Vertex>::untyped()
                    }
                }

                fn attribute() -> VertexAttribute {
                    VertexAttribute {
                        binding: Self::index(),
                        format: <Self::V as Vertex>::untyped(),
                        location: Self::index(),
                        offset: 0,
                    }
                }
            }
        }
    };
}

impl_vertex_attribute!(Position, positions, XYZ<f32>, POSITIONS);
impl_vertex_attribute!(Normal, normals, XYZ<Normalized<i8>>, NORMALS);
impl_vertex_attribute!(Tangent, tangents, XYZW<Normalized<i8>>, TANGENTS);
impl_vertex_attribute!(Color, colors, XYZ<Normalized<u8>>, COLORS);
impl_vertex_attribute!(TexCoord, uvs, XY<Normalized<u8>>, TEX_COORDS);