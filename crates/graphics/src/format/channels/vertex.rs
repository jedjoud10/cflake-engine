use crate::format::{AnyElement, Normalized};
use crate::pod::GpuPod;

// The channels that represent the vertices
pub struct X<T: AnyElement>(T);
pub struct XY<T: AnyElement>(vek::Vec2<T>);
pub struct XYZ<T: AnyElement>(vek::Vec3<T>);
pub struct XYZW<T: AnyElement>(vek::Vec4<T>);

// Used for vertex formats ONLY
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum VertexChannels {
    One,   // X
    Two,   // XY
    Three, // XYZ
    Four,  // XYZW
}

impl VertexChannels {
    // Count the number of channels that we have in total
    pub const fn count(&self) -> u32 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }
}
