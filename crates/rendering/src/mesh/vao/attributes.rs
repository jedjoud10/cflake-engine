use crate::{object::Shared, mesh::{VertexLayout, VertexAssembly}, buffer::ArrayBuffer};

use super::standard::StandardAttributeSet;


// Attribute base that will make up the elements of compound attributes.
pub trait ScalarAttribute: Shared {
    const GL_TYPE: u32;
}

// A compound attribute, like a vector (as in vec2, vec3, vec4) that consists of multiple attributes
pub trait RawAttribute: Shared {
    const GL_TYPE: u32;
    const COUNT_PER_VERTEX: u32;
}

// Base attribute implementaions
impl ScalarAttribute for f32 {
    const GL_TYPE: u32 = gl::FLOAT;
}

impl ScalarAttribute for i32 {
    const GL_TYPE: u32 = gl::INT;
}

impl ScalarAttribute for u32 {
    const GL_TYPE: u32 = gl::UNSIGNED_INT;
}

impl ScalarAttribute for i16 {
    const GL_TYPE: u32 = gl::SHORT;
}

impl ScalarAttribute for u16 {
    const GL_TYPE: u32 = gl::UNSIGNED_SHORT;
}

impl ScalarAttribute for i8 {
    const GL_TYPE: u32 = gl::BYTE;
}

impl ScalarAttribute for u8 {
    const GL_TYPE: u32 = gl::UNSIGNED_BYTE;
}

impl<T: ScalarAttribute> RawAttribute for T {
    const GL_TYPE: u32 = <T as ScalarAttribute>::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 1;
}

impl<T: ScalarAttribute> RawAttribute for vek::Vec2<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 2;
}

impl<T: ScalarAttribute> RawAttribute for vek::Vec3<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 3;
}

impl<T: ScalarAttribute> RawAttribute for vek::Vec4<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 4;
}

impl<T: ScalarAttribute> RawAttribute for vek::Rgb<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 3;
}

impl<T: ScalarAttribute> RawAttribute for vek::Rgba<T> {
    const GL_TYPE: u32 = T::GL_TYPE;
    const COUNT_PER_VERTEX: u32 = 4;
}

// A named attribute that has a specific name, like "Position", or "Normal"
pub trait Attribute {
    type Out: RawAttribute + Shared;
    const LAYOUT: VertexLayout;

    // Get the OpenGL array buffer from a specific attribute set
    fn get(set: &StandardAttributeSet) -> Option<&ArrayBuffer<Self::Out>>;
    fn get_mut(set: &mut StandardAttributeSet) -> Option<&mut ArrayBuffer<Self::Out>>;

    // Get the Rust vector from a vertex assembly (PS: The vector might be null)
    fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>>;
    fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>>;

    // Insert a vector into an assembly
    fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>);
}

pub mod named {
    use crate::mesh::vao::standard::StandardAttributeSet;

    use super::*;
    // Named attributes implement for empty structs
    pub struct Position;
    pub struct Normal;
    pub struct Tangent;
    pub struct Color;
    pub struct TexCoord0;

    impl Attribute for Position {
        type Out = vek::Vec3<f32>;
        const LAYOUT: VertexLayout = VertexLayout::POSITIONS;

        fn get(set: &StandardAttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
            set.positions.as_ref()
        }

        fn get_mut(set: &mut StandardAttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
            set.positions.as_mut()
        }

        fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>> {
            assembly.positions.as_ref()
        }

        fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>> {
            assembly.positions.as_mut()
        }

        fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>) {
            assembly.positions.insert(vec);
        }
    }

    impl Attribute for Normal {
        type Out = vek::Vec3<i8>;
        const LAYOUT: VertexLayout = VertexLayout::NORMALS;

        fn get(set: &StandardAttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
            set.normals.as_ref()
        }

        fn get_mut(set: &mut StandardAttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
            set.normals.as_mut()
        }

        fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>> {
            assembly.normals.as_ref()
        }

        fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>> {
            assembly.normals.as_mut()
        }

        fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>) {
            assembly.normals.insert(vec);
        }
    }

    impl Attribute for Tangent {
        type Out = vek::Vec4<i8>;
        const LAYOUT: VertexLayout = VertexLayout::TANGENTS;

        fn get(set: &StandardAttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
            set.tangents.as_ref()
        }

        fn get_mut(set: &mut StandardAttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
            set.tangents.as_mut()
        }

        fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>> {
            assembly.tangents.as_ref()
        }

        fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>> {
            assembly.tangents.as_mut()
        }

        fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>) {
            assembly.tangents.insert(vec);
        }
    }

    impl Attribute for Color {
        type Out = vek::Rgb<u8>;
        const LAYOUT: VertexLayout = VertexLayout::COLORS;

        fn get(set: &StandardAttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
            set.colors.as_ref()
        }

        fn get_mut(set: &mut StandardAttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
            set.colors.as_mut()
        }

        fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>> {
            assembly.colors.as_ref()
        }

        fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>> {
            assembly.colors.as_mut()
        }

        fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>) {
            assembly.colors.insert(vec);
        }
    }

    impl Attribute for TexCoord0 {
        type Out = vek::Vec2<u8>;
        const LAYOUT: VertexLayout = VertexLayout::TEX_COORD_0;

        fn get(set: &StandardAttributeSet) -> Option<&ArrayBuffer<Self::Out>> {
            set.tex_coord_0.as_ref()
        }

        fn get_mut(set: &mut StandardAttributeSet) -> Option<&mut ArrayBuffer<Self::Out>> {
            set.tex_coord_0.as_mut()
        }

        fn get_from_assembly(assembly: &VertexAssembly) -> Option<&Vec<Self::Out>> {
            assembly.tex_coord_0.as_ref()
        }

        fn get_from_assembly_mut(assembly: &mut VertexAssembly) -> Option<&mut Vec<Self::Out>> {
            assembly.tex_coord_0.as_mut()
        }

        fn insert(assembly: &mut VertexAssembly, vec: Vec<Self::Out>) {
            assembly.tex_coord_0.insert(vec);
        }
    }
}

pub mod output {
    use super::named::*;
    use super::Attribute;
    // Type aliases for the underlying vertex attribute data
    pub type VePos = <Position as Attribute>::Out;
    pub type VeNormal = <Normal as Attribute>::Out;
    pub type VeTangent = <Tangent as Attribute>::Out;
    pub type VeColor = <Color as Attribute>::Out;
    pub type VeTexCoord0 = <TexCoord0 as Attribute>::Out;
}
