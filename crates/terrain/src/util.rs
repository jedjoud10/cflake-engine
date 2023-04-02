use graphics::{BufferMode, BufferUsage, Buffer, Graphics, Texture3D, R, Texel, TextureMode, TextureUsage, SamplerSettings, TextureMipMaps, Texture, TriangleBuffer, Vertex, XYZW};
use rendering::{attributes, AttributeBuffer};

// Common types used througohut the crate
pub type TempVertices = Buffer<<XYZW<f32> as Vertex>::Storage>;
pub type TempTriangles = Buffer<[u32; 3]>;
pub type Vertices = AttributeBuffer<attributes::Position>;
pub type Triangles = TriangleBuffer<u32>;

// Create counters that will help us generate the vertices
pub fn create_counters(graphics: &Graphics, count: usize) -> Buffer<u32> {
    Buffer::zeroed(
        graphics,
        count,
        BufferMode::Dynamic,
        BufferUsage::STORAGE | BufferUsage::WRITE,
    )
    .unwrap()
}

// Create a 3D storage texture with null contents with the specified size
pub fn create_texture3d<T: Texel>(
    graphics: &Graphics,
    size: u32,
) -> Texture3D<T> {
    Texture3D::<T>::from_texels(
        graphics,
        None,
        vek::Extent3::broadcast(size),
        TextureMode::Dynamic,
        TextureUsage::STORAGE,
        SamplerSettings::default(),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}